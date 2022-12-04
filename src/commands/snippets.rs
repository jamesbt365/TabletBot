use core::panic;
use serenity::builder::{CreateEmbed, CreateApplicationCommandOption, CreateApplicationCommand};
use serenity::json::Value;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::prelude::Context;
use crate::structures::{State, Snippet};
use crate::commands::{arg, respond_ok};

use super::{respond_err, respond_embed, arg_opt};

pub(super) fn sync_snippets(state: &State, command: &mut CreateApplicationCommand) {
  let mut id_option = CreateApplicationCommandOption::default();
  id_option.name("id")
    .description("The snippet's id")
    .kind(CommandOptionType::String)
    .required(true);

  for snippet in state.snippets.iter().take(25) {
    let name = format!("{}: {}", snippet.id, snippet.title);
    id_option.add_string_choice(name, snippet.id.clone());
  }

  insert_option(command, 0, id_option);
}

pub(super) async fn snippet(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  match arg(interaction, "id") {
    CommandDataOptionValue::String(id) => {
      interaction.defer(ctx).await.expect("Failed to defer interaction");

      if let Some(snippet) = get_snippet(ctx, &id).await {
        let embed = snippet.create_embed();

        respond_embed(ctx, interaction, &embed, false).await;
      } else {
        respond_err(ctx, interaction, "Failed to find snippet", &format!("Failed to find the snippet '{id}'")).await;
      }
    },
    _ => panic!("Invalid arguments provided to command: {}", &interaction.data.name)
  }
}

pub(super) async fn edit_snippet(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  let id = arg(interaction, "id");
  let title = arg_opt(interaction, "title");
  let content = arg_opt(interaction, "content");

  if let CommandDataOptionValue::String(id) = id {
    interaction.defer(ctx).await.expect("Failed to defer interaction");

    {
      let mut data = ctx.data.write().await;
      let state = data.get_mut::<State>().expect("Failed to get state");

      let snippet = state.snippets.iter_mut().find(|s| s.id.eq(&id));

      if let Some(snippet) = snippet {
        if let Some(CommandDataOptionValue::String(title)) = title {
          snippet.title = title;
        }

        if let Some(CommandDataOptionValue::String(content)) = content {
          snippet.content = content;
        }

        println!("Snippet edited '{}: {}'", &snippet.title, &snippet.content);

        state.write()
      } else {
        match (title, content) {
          (
            Some(CommandDataOptionValue::String(title)),
            Some(CommandDataOptionValue::String(content))
          ) => {
            let snippet = Snippet {
              id: id.clone(),
              title: title.clone(),
              content: content.replace(r#"\n"#, "\n")
            };

            println!("New snippet created '{}: {}'", id, title);

            state.snippets.push(snippet);
            state.write()
          },
          _ => {
            let title = "Failed to edit snippet";
            let content = &format!("The snippet '{}' does not exist", &id);
            return respond_err(ctx, interaction, title, content).await
          }
        }
      }
    }

    super::update_commands(ctx).await;

    let mut embed = get_snippet(ctx, &id).await
      .expect("Failed to get snippet for recently modified snippet")
      .create_embed();

    embed.colour(super::OK_COLOUR);

    respond_embed(ctx, interaction, &embed, false).await;
  }
}

pub(super) async fn create_snippet(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  let id = arg(interaction, "id");
  let title = arg(interaction, "title");
  let content = arg(interaction, "content");

  match (id, title, content) {
    (
      CommandDataOptionValue::String(id),
      CommandDataOptionValue::String(title),
      CommandDataOptionValue::String(content)
    ) => {
      interaction.defer(ctx).await.expect("Failed to defer interaction");

      let embed = {
        let mut data = ctx.data.write().await;
        let state = data.get_mut::<State>().expect("Failed to get state");

        if let Some(snippet) = state.snippets.iter().position(|s| s.id.eq(&id)) {
          state.snippets.remove(snippet);
        }

        let snippet = Snippet {
          id: id.clone(),
          title: title.clone(),
          content: content.replace(r#"\n"#, "\n")
        };

        println!("New snippet created '{}: {}'", id, title);

        let mut embed = snippet.create_embed();
        embed.colour(super::OK_COLOUR);

        state.snippets.push(snippet);
        state.write();

        if state.snippets.len() > 25 {
          embed.field("Warning", "There are more than 25 snippets, some may not appear in the snippet list.", false);
        }

        embed
      };

      super::update_commands(ctx).await;
      respond_embed(ctx, interaction, &embed, false).await;
    },
    _ => panic!("Invalid arguments provided to command: {}", &interaction.data.name)
  }
}

pub(super) async fn remove_snippet(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  let id = arg(interaction, "id");

  match id {
    CommandDataOptionValue::String(id) => {
      interaction.defer(ctx).await.expect("Failed to defer interaction");

      println!("Removing snippet '{id}'");

      match get_snippet(ctx, &id).await {
        Some(snippet) => {
          rm_snippet(ctx, &snippet).await;
          super::update_commands(ctx).await;

          let title = &"Snippet successfully removed";
          let content = &&format!("Removed snippet '{}: {}'", snippet.id, snippet.title);
          respond_ok(ctx, interaction, title, content).await;
        },
        None => {
          let title = &"Failed to remove snippet";
          let content = &&format!("The snippet '{id}' does not exist");
          respond_err(ctx, interaction, title, content).await
        }
      }
    },
    _ => panic!("Invalid arguments provided to command: {}", interaction.data.name)
  }
}

pub(super) async fn export_snippet(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  let id = arg(interaction, "id");

  match id {
    CommandDataOptionValue::String(id) => {
      let snippet = get_snippet(ctx, &id).await
        .expect("Failed to get snippet");

      let embed = snippet.create_embed();

      let result = interaction.create_followup_message(ctx, |r| r
        .content(format!("```{}```", snippet.content.replace("\n", r#"\n"#)))
        .add_embed(embed)
      ).await;

      if let Err(e) = result {
        println!("Failed to respond to interaction: {} {}", interaction.data.name, e)
      }
    },
    _ => panic!("Invalid arguments provided to command: {}", interaction.data.name)
  }
}

trait SnippetEmbed {
  fn create_embed(&self) -> CreateEmbed;
}

impl SnippetEmbed for Snippet {
  fn create_embed(&self) -> CreateEmbed {
    CreateEmbed::default()
      .title(&self.title)
      .description(&self.content)
      .colour(super::ACCENT_COLOUR)
      .clone()
  }
}

async fn get_snippet(ctx: &Context, id: &str) -> Option<Snippet> {
  let data = ctx.data.read().await;
  let state = data.get::<State>().expect("Failed to get state");

  state.snippets.iter()
    .find(|s| s.id.eq(id))
    .cloned()
}

async fn rm_snippet(ctx: &Context, snippet: &Snippet) {
  let mut data = ctx.data.write().await;
  let state = data.get_mut::<State>()
    .expect("Failed to get state");

  let index = state.snippets.iter()
    .position(|s| s.id == snippet.id)
    .expect("Snippet was not found in vec");

  println!("Removing snippet '{}: {}'", snippet.id, snippet.title);
  state.snippets.remove(index);
  state.write();
}

fn insert_option(command: &mut CreateApplicationCommand, index: usize, option: CreateApplicationCommandOption) -> &mut CreateApplicationCommand {
  let new_option = serenity::json::hashmap_to_json_map(option.0);
  let options = command.0.entry("options").or_insert_with(|| Value::from(Vec::<Value>::new()));
  let opt_arr = options.as_array_mut().expect("Must be an array");
  opt_arr.insert(index, Value::from(new_option));

  command
}
