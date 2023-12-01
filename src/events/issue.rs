use ::serenity::builder::CreateEmbedAuthor;
use octocrab::models::issues::Issue;
use octocrab::models::pulls::PullRequest;
use poise::serenity_prelude::{self as serenity, Colour, Context, CreateEmbed, Message};
use regex::Regex;
use crate::{structures::Embeddable, Data};

const DEFAULT_REPO_OWNER: &str = "OpenTabletDriver";
const DEFAULT_REPO_NAME: &str = "OpenTabletDriver";

const OPEN_COLOUR: Colour = Colour::new(0x238636);
const RESOLVED_COLOUR: Colour = Colour::new(0x8957e5);
const CLOSED_COLOUR: Colour = Colour::new(0xda3633);

pub async fn message(data: &Data, ctx: &Context, message: &Message) {
    if let Some(embeds) = issue_embeds(data, message).await {
        let typing = message.channel_id.start_typing(&ctx.http);

        let content: serenity::CreateMessage = serenity::CreateMessage::default()
            .embeds(embeds)
            .reference_message(message);
        let _ = message.channel_id.send_message(ctx, content).await;

        typing.stop();
    }
}

async fn issue_embeds(data: &Data, message: &Message) -> Option<Vec<CreateEmbed>> {
    let mut embeds: Vec<CreateEmbed> = vec![];
    let client = octocrab::instance();
    let ratelimit = client.ratelimit();

    let regex = Regex::new(r#" ?([a-z]+)?#([0-9]+[0-9]) ?"#).expect("Expected numbers regex");

    let custom_repos = {data.state.lock().unwrap().issue_prefixes.clone()};

    let mut issues = client.issues(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME);
    let mut prs = client.pulls(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME);

    for capture in regex.captures_iter(&message.content) {
        if let Some(m) = capture.get(2) {
            let issue_num = m.as_str().parse::<u64>().expect("Match is not a number");

            if let Some(repo) = capture.get(1) {
                let repository = custom_repos.get(repo.as_str());
                if let Some(repository) = repository {
                    let (owner, repo) = repository.get();

                    issues = client.issues(owner, repo);
                    prs = client.pulls(owner, repo);
                }
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
