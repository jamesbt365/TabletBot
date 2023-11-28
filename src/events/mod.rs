use serenity::model::prelude::Message;

use poise::serenity_prelude as serenity;

use crate::{Data, Error};

pub mod code;
pub mod issue;

pub async fn message(
    ctx: &serenity::Context,
    new_message: Message,
    _data: &Data,
) -> Result<(), Error> {
    if !new_message.author.bot {
        issue::message(ctx, &new_message).await;
        code::message(ctx, &new_message).await;
    }
    Ok(())
}
