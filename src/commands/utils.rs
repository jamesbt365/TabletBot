use crate::{
    commands::{respond_embed, respond_err, respond_ok},
    structures::RepositoryDetails,
    Context, Error,
};

use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};
use regex::Regex;
use serenity::futures::{self, Stream, StreamExt};

async fn autocomplete_key<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let snippet_list: Vec<String> = {
        ctx.data()
            .state
            .lock()
            .unwrap()
            .issue_prefixes
            .iter()
            .take(25)
            .map(|s| s.0.clone())
            .collect()
    };

    futures::stream::iter(snippet_list)
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

/// Create an embed in the current channel.
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, guild_only)]
pub async fn embed(
    ctx: Context<'_>,
    #[description = "The embed title"] title: Option<String>,
    #[description = "The embed description"] description: Option<String>,
    #[description = "The color of the embed in hexidecimal form. (ex: ff00ff)"] color: Option<
        String,
    >,
    #[description = "The embed url"] url: Option<String>,
    #[description = "The embed image"] image: Option<String>,
    #[description = "The embed footer text"] footer: Option<String>,
    #[description = "The embed thumbnail"] thumbnail: Option<String>,
) -> Result<(), Error> {
    let at_least_one_property_set = title.is_some()
        || description.is_some()
        || image.is_some()
        || thumbnail.is_some()
        || footer.is_some();

    let url_invalid = url.is_some() && title.is_none();

    if !at_least_one_property_set {
        respond_err(
            &ctx,
            "Failed to respond with embed",
            "Please provide at least one title, description, image, footer or thumbnail",
        )
        .await;
        ctx.say("You must provide at least one title, description, image or thumbnail.")
            .await?;
        return Ok(());
    }

    if url_invalid {
        respond_err(
            &ctx,
            "Failed to respond with embed",
            "To set a url, you must set a title",
        )
        .await;
        return Ok(());
    }

    let mut embed = CreateEmbed::default();

    if let Some(title) = title {
        embed = embed.title(title);
    }

    if let Some(description) = description {
        embed = embed.description(description.replace(r"\n", "\n"));
    }
    if let Some(image) = image {
        embed = embed.image(image);
    }

    if let Some(footer) = footer {
        embed = embed.footer(CreateEmbedFooter::new(footer));
    }

    if let Some(thumbnail) = thumbnail {
        embed = embed.thumbnail(thumbnail);
    }

    if let Some(color) = color {
        match hex::decode(color.to_ascii_lowercase().replace('#', "")) {
            Ok(hex_arr) => {
                embed = embed.color(Colour::from_rgb(hex_arr[0], hex_arr[1], hex_arr[2]));
            }
            Err(e) => {
                let title = "Invalid color provided";
                let content = &format!(
                    "The color '{}' is not a valid hexadecimal color: {}",
                    &color, e
                );
                respond_err(&ctx, title, content).await;
            }
        }
    }

    respond_embed(&ctx, embed, false).await;

    Ok(())
}

/// Adds an issue token
#[poise::command(rename = "add-issue-token", slash_command, guild_only)]
pub async fn add_issue_token(
    ctx: Context<'_>,
    #[description = "The key to the issue token in a lowercase alphabetic string"] key: String,
    #[description = "The owner of the repository."] owner: String,
    #[description = "The respository name."] repository: String,
) -> Result<(), Error> {
    let key_regex = Regex::new(r"[a-z+]+$").unwrap();
    let repo_details_regex = Regex::new(r"^[a-zA-Z0-9](?:[a-zA-Z0-9.-]*[a-zA-Z0-9])?$").unwrap();
    if !key_regex.is_match(&key) {
        respond_err(
            &ctx,
            "Issue token parsing error",
            "The key is limited to lowercase letters only.",
        )
        .await;
        return Ok(());
    }
    if !repo_details_regex.is_match(&key) || !repo_details_regex.is_match(&repository) {
        respond_err(
            &ctx,
            "Issue token parsing error",
            "Your inputs for owner and repository name must be valid.",
        )
        .await;
        return Ok(());
    }

    {
        let mut mutex_guard = { ctx.data().state.lock().unwrap() };
        let details = RepositoryDetails {
            owner: owner.clone(),
            name: repository.clone(),
        };

        mutex_guard.issue_prefixes.insert(key.clone(), details);
        println!(
            "Successfully added issue token {} for **{}/{}**",
            key, owner, repository
        );
        mutex_guard.write();
    };

    respond_ok(
        &ctx,
        "Successfully added issue token",
        &format!("{}: {}/{}", key, owner, repository),
    )
    .await;

    Ok(())
}

/// Removes an issue token.
#[poise::command(rename = "remove-issue-token", slash_command, guild_only)]
pub async fn remove_issue_token(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_key"]
    #[description = "The issue token key."]
    key: String,
) -> Result<(), Error> {
    // I know we could just do rm_repo, but that doesn't return a result.
    // I may change this in the future, but before I do that I'll probably
    // impl a solution directly into the types?

    // not sure why I have to do this, it won't settle otherwise.
    let key_str = format!("The issue token with the key '{}' has been removed", key);
    match get_repo_details(&ctx, &key).await {
        Some(_) => {
            rm_repo(&ctx, &key).await;

            respond_ok(&ctx, "Successfully removed token!", &key_str).await;
        }
        None => {
            let title = "Failure to find issue token";
            let content = format!("The key '{}' does not exist.", key);
            respond_err(&ctx, title, &content).await;
        }
    };

    Ok(())
}

/// Lists all snippets
#[poise::command(
    rename = "list-tokens",
    slash_command,
    prefix_command,
    guild_only,
    track_edits
)]
pub async fn list_tokens(ctx: Context<'_>) -> Result<(), Error> {
    let tokens = { ctx.data().state.lock().unwrap().issue_prefixes.clone() };

    let mut embed = CreateEmbed::default()
        .title("Issue tokens")
        .color(Colour::TEAL);

    // fields are limited to 25 max, we can't display more than 25 snippets in the snippets command
    // due to a discord limitation.
    for token in tokens.iter().take(25) {
        embed = embed.field(
            format!("**{}**", token.0),
            format!("{}/{}", token.1.owner, token.1.name),
            false,
        );
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

async fn get_repo_details(ctx: &Context<'_>, key: &str) -> Option<RepositoryDetails> {
    let data = ctx.data();
    let mutex_guard = data.state.lock().unwrap();

    mutex_guard.issue_prefixes.get(key).cloned()
}

async fn rm_repo(ctx: &Context<'_>, key: &str) {
    let data = ctx.data();
    let mut mutex_guard = data.state.lock().unwrap();

    mutex_guard.issue_prefixes.remove(key);
}
