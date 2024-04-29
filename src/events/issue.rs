use std::time::Duration;

use crate::{commands::interaction_err, structures::Embeddable, Data};
use ::serenity::builder::CreateEmbedAuthor;
use octocrab::models::issues::Issue;
use octocrab::models::pulls::PullRequest;
use poise::serenity_prelude::{
    self as serenity, ButtonStyle, Colour, ComponentInteraction, Context, CreateActionRow,
    CreateButton, CreateEmbed, CreateInteractionResponse, Message,
};
use regex::Regex;

const DEFAULT_REPO_OWNER: &str = "OpenTabletDriver";
const DEFAULT_REPO_NAME: &str = "OpenTabletDriver";

const OPEN_COLOUR: Colour = Colour::new(0x238636);
const RESOLVED_COLOUR: Colour = Colour::new(0x8957e5);
const CLOSED_COLOUR: Colour = Colour::new(0xda3633);

pub async fn message(data: &Data, ctx: &Context, message: &Message) {
    if let Some(embeds) = issue_embeds(data, message).await {
        let typing = message.channel_id.start_typing(&ctx.http);

        let ctx_id = message.id.get(); // poise context isn't available here.
        let remove_id = format!("{}remove", ctx_id);
        let hide_body_id = format!("{}hide_body", ctx_id);
        let remove = CreateActionRow::Buttons(vec![CreateButton::new(&remove_id)
            .label("delete")
            .style(ButtonStyle::Danger)]);

        let components = serenity::CreateActionRow::Buttons(vec![
            CreateButton::new(&remove_id)
                .label("delete")
                .style(ButtonStyle::Danger),
            CreateButton::new(&hide_body_id).label("hide body"),
        ]);

        let content: serenity::CreateMessage = serenity::CreateMessage::default()
            .embeds(embeds)
            .reference_message(message)
            .components(vec![components]);
        let msg_result = message.channel_id.send_message(ctx, content).await;
        typing.stop();

        let mut msg_deleted = false;
        let mut body_hid = false;
        while let Some(press) = serenity::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
            .timeout(Duration::from_secs(60))
            .await
        {
            let has_perms = check_perms(ctx, &press).await;

            if press.data.custom_id == remove_id {
                if press.user.id == message.author.id || has_perms {
                    let _ = press
                        .create_response(ctx, CreateInteractionResponse::Acknowledge)
                        .await;
                    if let Ok(ref msg) = msg_result {
                        let _ = msg.delete(ctx).await;
                    }
                    msg_deleted = true;
                } else {
                    interaction_err(
                        ctx,
                        &press,
                        "Unable to use interaction because you are missing `MANAGE_MESSAGES`.",
                    )
                    .await;
                }
            }

            if press.data.custom_id == hide_body_id {
                if press.user.id == message.author.id {
                    if !body_hid {
                        let mut hid_body_embeds: Vec<CreateEmbed> = Vec::new();
                        if let Ok(ref msg) = msg_result {
                            for mut embed in msg.embeds.clone() {
                                embed.description = None;
                                let embed: CreateEmbed = embed.clone().into();
                                hid_body_embeds.push(embed);
                            }
                        }

                        let _ = press
                            .create_response(
                                ctx,
                                serenity::CreateInteractionResponse::UpdateMessage(
                                    serenity::CreateInteractionResponseMessage::new()
                                        .embeds(hid_body_embeds)
                                        .components(vec![remove.clone()]),
                                ),
                            )
                            .await;
                    }
                    body_hid = true;
                } else {
                    interaction_err(
                        ctx,
                        &press,
                        "Unable to use interaction because you are missing `MANAGE_MESSAGES`.",
                    )
                    .await;
                }
            }
        }
        // Triggers on timeout.
        if !msg_deleted {
            if let Ok(mut msg) = msg_result {
                let _ = msg
                    .edit(ctx, serenity::EditMessage::default().components(vec![]))
                    .await;
            }
        }
    }
}

async fn check_perms(ctx: &Context, press: &ComponentInteraction) -> bool {
    let Some(ref member) = press.member else {
        return false;
    };

    let Some(guild_id) = press.guild_id else {
        return false;
    };
    let Some(guild) = ctx.cache.guild(guild_id) else {
        return false;
    };
    let Some((_, channel)) = guild.channels.iter().find(|c| c.0 == &press.channel_id) else {
        return false;
    };
    let permissions = guild.user_permissions_in(channel, member);

    permissions.manage_messages()
}

async fn issue_embeds(data: &Data, message: &Message) -> Option<Vec<CreateEmbed>> {
    let mut embeds: Vec<CreateEmbed> = vec![];
    let client = octocrab::instance();
    let ratelimit = client.ratelimit();

    let regex = Regex::new(r#" ?([a-zA-Z0-9-_.]+)?#([0-9]+) ?"#).expect("Expected numbers regex");

    let custom_repos = { data.state.read().unwrap().issue_prefixes.clone() };

    let mut issues = client.issues(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME);
    let mut prs = client.pulls(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME);

    for capture in regex.captures_iter(&message.content) {
        if let Some(m) = capture.get(2) {
            let issue_num = m.as_str().parse::<u64>().expect("Match is not a number");

            if let Some(repo) = capture.get(1) {
                let repository = custom_repos.get(&repo.as_str().to_lowercase());
                if let Some(repository) = repository {
                    let (owner, repo) = repository.get();

                    issues = client.issues(owner, repo);
                    prs = client.pulls(owner, repo);
                } else {
                    continue; // discards when it doesn't match a repo.
                };
            }

            let ratelimit = ratelimit
                .get()
                .await
                .expect("Failed to get github rate limit");

            if ratelimit.rate.remaining > 2 {
                if let Ok(pr) = prs.get(issue_num).await {
                    embeds.push(pr.embed());
                } else if let Ok(issue) = issues.get(issue_num).await {
                    embeds.push(issue.embed());
                }
            }
        }
    }

    if embeds.is_empty() {
        None
    } else {
        Some(embeds)
    }
}

trait Document {
    fn get_title(&self) -> String;
    fn get_content(&self) -> String;
    fn get_colour(&self) -> Colour;
    fn get_labels(&self) -> Option<String>;
}

impl Embeddable for Issue {
    fn embed(&self) -> CreateEmbed {
        let default = CreateEmbed::default();
        let author = CreateEmbedAuthor::new(&self.user.login)
            .url(self.user.url.clone())
            .icon_url(self.user.avatar_url.clone());
        let mut embed = default
            .title(self.get_title())
            .description(self.get_content())
            .url(self.html_url.as_str())
            .colour(self.get_colour())
            .author(author);

        if let Some(milestone) = &self.milestone {
            embed = embed.field("Milestone", &milestone.title, true);
        }

        if let Some(labels) = self.get_labels() {
            embed = embed.field("Labels", labels, true);
        }

        embed
    }
}

impl Document for Issue {
    fn get_title(&self) -> String {
        format!("#{}: {}", self.number, self.title)
    }

    fn get_content(&self) -> String {
        let body = self.body.as_deref().unwrap_or_default();

        let mut description = String::default();
        for line in body.split('\n').take(15) {
            description.push_str(&format!("{}\n", line));
        }

        description.shrink_to(4096);
        description
    }

    fn get_colour(&self) -> Colour {
        match self.closed_at {
            Some(_) => CLOSED_COLOUR,
            None => OPEN_COLOUR,
        }
    }

    fn get_labels(&self) -> Option<String> {
        if !self.labels.is_empty() {
            let labels = &self
                .labels
                .iter()
                .map(|l| l.name.clone())
                .collect::<Vec<String>>();

            Some(format!("`{}`", labels.join("`, `")))
        } else {
            None
        }
    }
}

impl Embeddable for PullRequest {
    fn embed(&self) -> CreateEmbed {
        let mut description = self.body.clone().unwrap_or_default();
        description.shrink_to(4096);

        let default = CreateEmbed::default();
        let mut embed = default
            .title(self.get_title())
            .description(self.get_content())
            .colour(self.get_colour());

        if let Some(user) = &self.user {
            let author = CreateEmbedAuthor::new(user.login.clone())
                .url(user.url.clone())
                .icon_url(user.avatar_url.clone());
            embed = embed.author(author);
        }

        if let Some(url) = &self.html_url {
            embed = embed.url(url.as_str());
        }

        if let Some(milestone) = &self.milestone {
            embed = embed.field("Milestone", &milestone.title, true);
        }

        if let Some(labels) = self.get_labels() {
            embed = embed.field("Labels", labels, true);
        }

        embed.to_owned()
    }
}

impl Document for PullRequest {
    fn get_title(&self) -> String {
        match &self.title {
            Some(title) => format!("#{}: {}", self.number, title),
            None => format!("#{}", self.number),
        }
    }

    fn get_content(&self) -> String {
        let body = self.body.as_deref().unwrap_or_default();

        let mut content = String::default();
        for line in body.split('\n').take(15) {
            content.push_str(&format!("{}\n", line));
        }

        content.shrink_to(4096);
        content
    }

    fn get_colour(&self) -> Colour {
        match self.closed_at {
            Some(_) => match self.merged_at {
                Some(_) => RESOLVED_COLOUR,
                None => CLOSED_COLOUR,
            },
            None => OPEN_COLOUR,
        }
    }

    fn get_labels(&self) -> Option<String> {
        if let Some(labels) = &self.labels {
            if !labels.is_empty() {
                let labels = labels
                    .iter()
                    .map(|l| l.name.clone())
                    .collect::<Vec<String>>();

                return Some(format!("`{}`", labels.join("`, `")));
            }
        }

        None
    }
}
