use serenity::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommandOption};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::prelude::Context;
use serenity::utils::Colour;
use crate::commands::types::*;
use crate::structures::{Container, Snippet};

use super::register;

fn get_str_opt(command: &ApplicationCommandInteraction, slice: usize) -> Option<String> {
  let opt = command.data.options.get(slice)
    .expect(&format!("Expected argument at {}", slice))
    .resolved
    .as_ref()
    .expect(&format!("Expected string for argument value at {}", slice));

  if let CommandDataOptionValue::String(s) = opt {
    Some(s.clone())
  } else {
    None
  }
}

async fn followup_snippet(ctx: &Context, container: &Container, command: &ApplicationCommandInteraction, snippet_name: String) {
  let snippet = container.state.snippets
    .iter()
    .find(|s| snippet_name.eq(&s.id));

  if let Some(snippet) = snippet {
    command.create_followup_message(&ctx.http, |f|
      f.embed(|e| e
        .title(&snippet.title)
        .description(&snippet.content)
        .color(OK_COLOR)
      )
    ).await.expect("Failed to respond with snippet embed");
  } else {
    command.create_followup_message(&ctx.http, |f|
      f.embed(|e| e
        .title("Error")
        .description(format!("Failed to get embed for snippet '{}'", snippet_name))
        .color(ERROR_COLOR)
      )
    ).await.expect("Failed to respond with error embed");
  }
}

trait SnippetOpts {
  fn gen_opts(&mut self, container: &Container) -> &mut Self;
}

impl SnippetOpts for CreateApplicationCommandOption {
  fn gen_opts(&mut self, container: &Container) -> &mut Self {
    for snippet in container.state.snippets[..25].iter() {
      let name = format!("{}: {}", snippet.id, snippet.title);
      self.add_string_choice(name, snippet.id.clone());
    }

    self
  }
}

const OK_COLOR: Colour = Colour(0x2ecc71);
const ERROR_COLOR: Colour = Colour(0xe74c3c);

pub const SNIPPET_NAME: &str = "snippet";
pub struct SnippetCommand;

pub const SET_SNIPPET_NAME: &str = "set-snippet";
pub struct SetSnippetCommand;

pub const REMOVE_SNIPPET_NAME: &str = "remove-snippet";
pub struct RemoveSnippetCommand;

pub const EXPORT_SNIPPET_NAME: &str = "export-snippet";
pub struct ExportSnippetCommand;

#[async_trait]
impl SlashCommand for SnippetCommand {
  async fn register(ctx: &Context) -> CreateApplicationCommand {
    let data = ctx.data.read().await;
    let container = data.get::<Container>()
      .expect("Failed to get main container");

    CreateApplicationCommand::default()
      .name(SNIPPET_NAME)
      .description("Shows a snippet")
      .create_option(|option| option
        .name("snippet")
        .description("The name of the snippet")
        .kind(CommandOptionType::String)
        .required(true)
        .gen_opts(container)
      )
      .to_owned()
  }

  async fn invoke(ctx: &Context, command: &ApplicationCommandInteraction) {
    let id = get_str_opt(command, 0).expect("Expected id parameter");

    command.defer(&ctx.http).await.expect("Failed to defer interaction");

    let data = ctx.data.read().await;
    let container = data.get::<Container>()
      .expect("Failed to get main container");
    followup_snippet(ctx, container, command, id).await;
  }
}

#[async_trait]
impl SlashCommand for SetSnippetCommand {
  async fn register(_: &Context) -> CreateApplicationCommand {
    CreateApplicationCommand::default()
      .name(SET_SNIPPET_NAME)
      .description("Set a snippet's contents")
      .create_option(|o| o
        .name("id")
        .description("The snippet ID")
        .kind(CommandOptionType::String)
        .required(true)
      )
      .create_option(|o| o
        .name("title")
        .description("The embed title")
        .kind(CommandOptionType::String)
        .required(true)
      )
      .create_option(|o| o
        .name("content")
        .description("The embed content")
        .kind(CommandOptionType::String)
        .required(true)
      )
      .to_owned()
  }

  async fn invoke(ctx: &Context, command: &ApplicationCommandInteraction) {
    let id = get_str_opt(command, 0).expect("Expected id parameter");
    let title = get_str_opt(command, 1).expect("Expected title parameter");
    let content = get_str_opt(command, 2).expect("Expected content parameter").replace(r#"\n"#, "\n");

    command.defer(&ctx.http).await.expect("Failed to defer interaction");

    {
      let mut data = ctx.data.write().await;
      let container = data.get_mut::<Container>().expect("Failed to get main container");
      let old_snippet = container.state.snippets.iter_mut()
        .find(|s| s.id == id);

      match old_snippet {
        Some(o) => {
          o.title = title;
          o.content = content;
        },
        None => container.state.snippets.push(Snippet {
          id: id.clone(),
          title,
          content
        })
      }

      container.write();

      if let Err(err) = register(ctx).await {
        command.create_followup_message(&ctx.http, |f| f
          .embed(|e| e
            .title("Error")
            .description(format!("{}", err))
            .colour(ERROR_COLOR)
          )
        ).await.expect("Failed to reply with error");
      } else {
        followup_snippet(ctx, container, command, id).await;
      }
    }
  }
}

#[async_trait]
impl SlashCommand for RemoveSnippetCommand {
  async fn register(ctx: &Context) -> CreateApplicationCommand {
    let data = ctx.data.read().await;
    let container = data.get::<Container>().expect("Failed to get main container");

    CreateApplicationCommand::default()
      .name(REMOVE_SNIPPET_NAME)
      .description("Removes a snippet")
      .create_option(|o| o
        .name("id")
        .description("The snippet's ID")
        .kind(CommandOptionType::String)
        .required(true)
        .gen_opts(container)
      )
      .to_owned()
  }

  async fn invoke(ctx: &Context, command: &ApplicationCommandInteraction) {
    let id = get_str_opt(command, 0).expect("Expected id parameter");

    command.defer(&ctx.http).await.expect("Failed to defer interaction");

    let mut data = ctx.data.write().await;
    let container = data.get_mut::<Container>().expect("Failed to get main container");

    let index = container.state.snippets.iter().position(|s| s.id == id);

    match index.clone() {
      Some(i) => {
        container.state.snippets.remove(i);

        command.create_followup_message(&ctx.http, |r| r
          .embed(|e| e
            .title("Removed snippet")
            .description(format!("Successfully removed the '{}' snippet", id))
            .color(OK_COLOR)
          )
        ).await.expect("Failed to follow up interaction");
      },
      None => {
        command.create_followup_message(&ctx.http, |r| r
          .embed(|e| e
            .title("Error")
            .description(&format!("Failed to find a snippet '{}'", id))
            .color(ERROR_COLOR)
          )
        ).await.expect("Failed to follow up interaction");
      }
    };
  }
}

#[async_trait]
impl SlashCommand for ExportSnippetCommand {
  async fn register(_: &Context) -> CreateApplicationCommand {
    CreateApplicationCommand::default()
      .name(EXPORT_SNIPPET_NAME)
      .description("Exports a snippet for editing")
      .create_option(|o| o
        .name("id")
        .description("The snippet's ID")
        .kind(CommandOptionType::String)
        .required(true)
      )
      .to_owned()
  }

  async fn invoke(ctx: &Context, command: &ApplicationCommandInteraction) {
    let id = get_str_opt(command, 0).expect("Expected id parameter");

    command.defer(&ctx.http).await.expect("Failed to defer interaction");

    let data = ctx.data.read().await;
    let container = data.get::<Container>().expect("Failed to get main container");

    let snippet = container.state.snippets
      .iter()
      .find(|s| id.eq(&s.id));

    if let Some(s) = snippet {
      command.create_followup_message(&ctx.http, |f| f
        .content(format!("```\n{}\n```", s.content.replace(r#"\n"#, "\n")))
        .embed(|e| e
          .title(&s.title)
          .description(&s.content)
          .color(OK_COLOR)
        )
      ).await.expect("Failed to reply with snippet contents");
    } else {
      command.create_followup_message(&ctx.http, |f| f
        .embed(|e| e
          .title("Error")
          .description(format!("Failed to find snippet '{}'", id))
        )
      ).await.expect("Failed to reply with error embed");
    }
  }
}
