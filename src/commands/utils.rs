use serenity::builder::CreateEmbed;
use serenity::model::prelude::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::prelude::Context;
use serenity::utils::Colour;

use super::{arg_opt, respond_err, respond_embed};

pub(super) async fn embed(ctx: &Context, interaction: &ApplicationCommandInteraction) {
  let title = arg_opt(interaction, "title");
  let description = arg_opt(interaction, "description");
  let color = arg_opt(interaction, "color");
  let url = arg_opt(interaction, "url");
  let footer_text = arg_opt(interaction, "footer");
  let image = arg_opt(interaction, "image");

  interaction.defer(ctx).await.expect("Failed to defer interaction");

  let mut embed = CreateEmbed::default();

  if let Some(CommandDataOptionValue::String(title)) = title {
    embed.title(title);
  }

  if let Some(CommandDataOptionValue::String(description)) = description {
    embed.description(description);
  }

  if let Some(CommandDataOptionValue::String(color)) = &color {
    match hex::decode(color.to_ascii_lowercase().replace("#",  "")) {
      Ok(hex_arr) => {
        embed.color(Colour::from_rgb(hex_arr[0], hex_arr[1], hex_arr[2]));
      },
      Err(e) => {
        let title = "Invalid color provided";
        let content = &format!("The color '{}' is not a valid hexadecimal color: {}", &color, e);
        return respond_err(ctx, interaction, title, content).await
      }
    }
  }

  if let Some(CommandDataOptionValue::String(url)) = url {
    match url.parse::<reqwest::Url>() {
      Ok(_) => {
        if embed.0.contains_key("title") {
          embed.url(url);
        } else {
          let title = "Invalid parameters";
          let content = "A title is required for a url to function";
          return respond_err(ctx, interaction, title, content).await
        }
      },
      Err(e) => {
        let title = "Invalid url provided";
        let content = &format!("The url '{}' is not a valid url: {}", url, e);
        return respond_err(ctx, interaction, title, content).await
      }
    }
  }

  if let Some(CommandDataOptionValue::String(footer_text)) = footer_text {
    embed.footer(|f| f.text(footer_text));
  }

  if let Some(CommandDataOptionValue::String(image)) = image {
    match image.parse::<reqwest::Url>() {
      Ok(_) => {
        embed.image(image);
      },
      Err(e) => {
        let title = "Invalid image url provided";
        let content = &format!("The image url '{}' is not a valid image url: {}", image, e);

        return respond_err(ctx, interaction, title, content).await
      }
    }
  }

  if embed.0.contains_key("title") || embed.0.contains_key("description") || embed.0.contains_key("footer") {
    respond_embed(ctx, interaction, &embed, false).await
  } else {
    respond_err(ctx, interaction, "Failed to respond with embed", "Embed does not have any content").await
  }
}
