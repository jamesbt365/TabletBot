use serenity::builder::CreateApplicationCommand;
use serenity::builder::CreateApplicationCommandOption;
use serenity::builder::CreateEmbed;
use serenity::http::Http;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::prelude::Context;
use serenity::prelude::TypeMapKey;
use serenity::utils::Colour;
use std::collections::HashMap;
use crate::structures::State;

mod snippets;

pub async fn register(ctx: &Context) -> ApplicationCommandMap {
  println!("Registering slash commands...");

  let mut data = ctx.data.write().await;
  let state = data.get_mut::<State>()
    .expect("Failed to get state");

  let commands = ApplicationCommandMap::new(state);

  match commands.register(ctx).await {
    Ok(c) => println!("Registered {} slash commands", c.len()),
    Err(e) => println!("Failed to register slash commands: {}", e)
  }

  commands
}

pub async fn interact(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  match interaction.data.name.as_str() {
    "snippet" => snippets::snippet(ctx, interaction).await,
    "create-snippet" => snippets::create_snippet(ctx, interaction).await,
    "edit-snippet" => snippets::edit_snippet(ctx, interaction).await,
    "remove-snippet" => snippets::remove_snippet(ctx, interaction).await,
    "export-snippet" => snippets::export_snippet(ctx, interaction).await,
    _ => println!("WARNING: Received invalid application command interaction!: {}", interaction.data.name)
  }
}

pub(crate) const ACCENT_COLOUR: Colour = Colour(0x8957e5);
pub(crate) const OK_COLOUR: Colour = Colour(0x2ecc71);
pub(crate) const ERROR_COLOUR: Colour = Colour(0xe74c3c);

type CommandHashMap = HashMap<&'static str, CreateApplicationCommand>;

#[derive(Clone)]
pub struct ApplicationCommandMap(pub CommandHashMap);

impl TypeMapKey for ApplicationCommandMap {
  type Value = ApplicationCommandMap;
}

impl ApplicationCommandMap {
  pub fn new(state: &State) -> ApplicationCommandMap {
    let mut id_opt = CreateApplicationCommandOption::default();
    id_opt.name("id")
      .description("The snippet's id")
      .kind(CommandOptionType::String)
      .required(true);

    let mut title_opt = CreateApplicationCommandOption::default();
    title_opt.name("title")
      .description("The snippet's title")
      .kind(CommandOptionType::String);

    let mut content_opt = CreateApplicationCommandOption::default();
    content_opt.name("content")
      .description("The snippet's content")
      .kind(CommandOptionType::String);

    let snippet = CreateApplicationCommand::default()
      .description("Shows a snippet")
      .clone();

    let create_snippet = CreateApplicationCommand::default()
      .description("Creates a snippet")
      .add_option(id_opt)
      .add_option(title_opt.required(true).clone())
      .add_option(content_opt.required(true).clone())
      .clone();

    let edit_snippet = CreateApplicationCommand::default()
      .description("Edits a snippet")
      .add_option(title_opt.required(false).clone())
      .add_option(content_opt.required(false).clone())
      .clone();

    let remove_snippet = CreateApplicationCommand::default()
      .description("Removes a snippet")
      .clone();

    let export_snippet = CreateApplicationCommand::default()
      .description("Exports a snippet for user editing")
      .clone();

    let mut commands = ApplicationCommandMap(CommandHashMap::new());

    commands.insert("snippet", snippet);
    commands.insert("create-snippet", create_snippet);
    commands.insert("edit-snippet", edit_snippet);
    commands.insert("remove-snippet", remove_snippet);
    commands.insert("export-snippet", export_snippet);

    for (name, command) in commands.0.iter_mut() {
      match *name {
        "snippet" => snippets::sync_snippets(state, command),
        "remove-snippet" => snippets::sync_snippets(state, command),
        "export-snippet" => snippets::sync_snippets(state, command),
        "edit-snippet" => snippets::sync_snippets(state, command),
        _ => ()
      }
    }

    commands
  }

  fn insert(&mut self, k: &'static str, v: CreateApplicationCommand) -> Option<CreateApplicationCommand> {
    self.0.insert(k, v)
  }

  fn builders(&self) -> Vec<CreateApplicationCommand> {
    self.0.iter()
      .map(|p| {
        let mut builder = p.1.clone();
        builder.name(p.0);
        builder
      })
      .collect::<Vec<CreateApplicationCommand>>()
  }

  pub async fn register(&self, http: impl AsRef<Http>) -> Result<Vec<Command>, serenity::Error> {
    Command::set_global_application_commands(http, |commands| {
      commands.set_application_commands(self.builders())
    }).await
  }
}

pub fn arg(interaction: &ApplicationCommandInteraction, name: &'static str) -> CommandDataOptionValue {
  arg_opt(interaction, name).expect(&format!("No '{name}' argument provided")).clone()
}

pub fn arg_opt(interaction: &ApplicationCommandInteraction, name: &'static str) -> Option<CommandDataOptionValue> {
  let opt = interaction.data.options.iter()
    .find(|o| o.name == name);

  if let Some(opt) = opt {
    opt.resolved.as_ref().cloned()
  } else {
    None
  }
}

pub async fn respond_embed(ctx: &Context, interaction: &ApplicationCommandInteraction, embed: &CreateEmbed, ephemeral: bool) {
  let result = interaction.create_followup_message(ctx, |r| r
    .add_embed(embed.clone())
    .ephemeral(ephemeral)
  ).await;

  if let Err(e) = result {
    println!("Failed to respond to interaction: {} {}", interaction.data.name, e)
  }
}

pub async fn respond_ok(ctx: &Context, interaction: &ApplicationCommandInteraction, title: &str, content: &str) {
  let mut embed = CreateEmbed::default();
  let embed = embed
    .title(title)
    .description(content)
    .colour(OK_COLOUR);

  respond_embed(ctx, interaction, embed, false).await;
}

pub async fn respond_err(ctx: &Context, interaction: &ApplicationCommandInteraction, title: &str, content: &str) {
  let mut embed = CreateEmbed::default();
  let embed = embed
    .title(title)
    .description(content)
    .colour(ERROR_COLOUR);

  respond_embed(ctx, interaction, embed, false).await;
}

pub async fn update_commands(ctx: &Context) {
  let mut data = ctx.data.write().await;
  let state = data.get_mut::<State>()
    .expect("Failed to get state");

  println!("Updating commands...");
  let command_map = ApplicationCommandMap::new(state);

  println!("Registering newly updated commands");
  match command_map.register(ctx).await {
    Ok(commands) => println!("Successfully updated {} commands", commands.len()),
    Err(e) => println!("Failed to update commands: {e}")
  }
}
