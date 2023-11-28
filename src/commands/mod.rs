pub mod snippets;
pub mod utils;

pub(crate) const ACCENT_COLOUR: Colour = Colour(0x8957e5);
pub(crate) const OK_COLOUR: Colour = Colour(0x2ecc71);
pub(crate) const ERROR_COLOUR: Colour = Colour(0xe74c3c);

use serenity::model::Colour;

use crate::{Context, Error};

use poise::serenity_prelude::{self as serenity, CreateEmbed};

#[poise::command(prefix_command, hide_in_help)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

pub async fn respond_embed(ctx: &Context<'_>, embed: CreateEmbed, ephemeral: bool) {
    let builder = poise::CreateReply::default()
        .embed(embed)
        .ephemeral(ephemeral);
    let result = ctx.send(builder).await;

    if let Err(e) = result {
        println!("Failed to respond: {}", e)
    }
}

pub async fn respond_ok(ctx: &Context<'_>, title: &str, content: &str) {
    let embed = CreateEmbed::default()
        .title(title)
        .description(content)
        .colour(OK_COLOUR);

    respond_embed(ctx, embed, false).await;
}

pub async fn respond_err(ctx: &Context<'_>, title: &str, content: &str) {
    let embed = CreateEmbed::default()
        .title(title)
        .description(content)
        .colour(ERROR_COLOUR);

    respond_embed(ctx, embed, false).await;
}
