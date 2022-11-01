use core::panic;
use serenity::builder::CreateApplicationCommand;
use serenity::Error;
use serenity::http::Http;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use std::sync::Arc;
use crate::commands::snippets::*;
use crate::commands::types::*;

pub mod snippets;
pub mod types;

pub async fn get_commands(ctx: &Context) -> Vec<CreateApplicationCommand> {
  vec![
    SnippetCommand::register(ctx).await,
    SetSnippetCommand::register(ctx).await,
    RemoveSnippetCommand::register(ctx).await,
    ExportSnippetCommand::register(ctx).await
  ]
}

pub async fn register(ctx: &Context) -> Result<Vec<Command>, Error> {
  let commands = get_commands(ctx).await;

  Command::set_global_application_commands(&ctx.http, |c|
    c.set_application_commands(commands)
  ).await
}

pub async fn unregister(http: &Arc<Http>) -> Result<(), Error> {
  if let Ok(commands) = Command::get_global_application_commands(http).await {
    for command in commands {
      if let Err(e) = Command::delete_global_application_command(http, command.id).await {
        return Err(e);
      }
    }
  }

  Ok(())
}

pub async fn interact(ctx: &Context, command: &ApplicationCommandInteraction) {
  match command.data.name.as_str() {
    SNIPPET_NAME => SnippetCommand::invoke(ctx, command),
    SET_SNIPPET_NAME => SetSnippetCommand::invoke(ctx, command),
    REMOVE_SNIPPET_NAME => RemoveSnippetCommand::invoke(ctx, command),
    EXPORT_SNIPPET_NAME => ExportSnippetCommand::invoke(ctx, command),
    _ => panic!("Invalid interaction command: {}", command.data.name)
  }.await
}
