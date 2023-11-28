pub(crate) mod commands;
pub(crate) mod events;
pub(crate) mod formatting;
pub(crate) mod structures;

use std::sync::Mutex;
use std::time::Duration;
use std::{env, sync::Arc};

use octocrab::Octocrab;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use structures::SnippetState;

pub struct Data {
    pub octocrab: Arc<Octocrab>,
    pub snip: Mutex<SnippetState>,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let discord_token = env::var("DISCORD_TOKEN").expect("Expected discord api token");
    let github_token = env::var("GITHUB_TOKEN").expect("Expected github api token");

    let octo_builder = Octocrab::builder().personal_token(github_token);

    let octocrab = octocrab::initialise(octo_builder).expect("Failed to build github client");

    let snip = Mutex::new(structures::SnippetState::read());

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::register(),
            commands::snippets::snippet(),
            commands::snippets::create_snippet(),
            commands::snippets::delete_snippet(),
            commands::snippets::export_snippet(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(600))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),

        skip_checks_for_owners: false,
        event_handler: |event: &serenity::FullEvent, framework, data| {
            Box::pin(event_handler(event.clone(), framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::new(options, move |ctx, ready, framework| {
        Box::pin(async move {
            println!("Logged in as {}", ready.user.name);
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data { octocrab, snip })
        })
    });

    // pre post command stuff

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::Client::builder(discord_token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
}

pub async fn event_handler(
    event: serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    #[allow(clippy::single_match)]
    match event {
        serenity::FullEvent::Message { ctx, new_message } => {
            events::message(&ctx, new_message, data).await?;
        }
        _ => (),
    }

    Ok(())
}
