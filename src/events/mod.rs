use serenity::model::prelude::Message;
use serenity::prelude::Context;

pub mod issue;
pub mod code;

pub async fn message(ctx: &Context, msg: &Message) {
  issue::message(&ctx, &msg).await;
  code::message(&ctx, &msg).await;
}
