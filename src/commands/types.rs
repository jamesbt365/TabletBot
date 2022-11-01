use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[async_trait]
pub trait SlashCommand {
  async fn register(ctx: &Context) -> CreateApplicationCommand;
  async fn invoke(ctx: &Context, command: &ApplicationCommandInteraction);
}

pub trait MessageCommand {
  fn invoke(ctx: &mut Context, msg: &Message);
}

