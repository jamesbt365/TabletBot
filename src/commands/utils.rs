use crate::{
    commands::{respond_embed, respond_err},
    structures::RepositoryDetails,
    Context, Error,
};

use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};

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
#[poise::command(rename = "add-issue-token", slash_command, prefix_command, guild_only)]
pub async fn add_issue_token(
    ctx: Context<'_>,
    #[description = "The snippet's id"] key: String,
    #[description = "The snippet's title"] owner: String,
    #[description = "The snippet's content"] repository: String,
) -> Result<(), Error> {
    let mut mutex_guard = { ctx.data().state.lock().unwrap() };

    let details = RepositoryDetails {
        owner,
        name: repository,
    };

    mutex_guard.issue_prefixes.insert(key, details);

    mutex_guard.write();

    Ok(())
}
