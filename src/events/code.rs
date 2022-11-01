use std::path::Path;
use std::str::FromStr;
use regex::{Regex, Match};
use serenity::builder::CreateEmbed;
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use serenity::utils::Colour;
use crate::formatting::*;

const ACCENT_COLOUR: Colour = Colour(0x8957e5);

pub async fn message(ctx: &Context, message: &Message) {
  if let Some(embeds) = get_embeds(ctx, message).await {
    message.channel_id.send_message(&ctx.http, |f| f
      .add_embeds(embeds)
    ).await.expect("Failed to reply to code message");
  }
}

async fn get_embeds(ctx: &Context, message: &Message) -> Option<Vec<CreateEmbed>> {
  let typing = message.channel_id.start_typing(&ctx.http).expect("Failed to start typing");
  let mut embeds: Vec<CreateEmbed> = vec![];

  if let Some(refs) = FileReference::try_from_str(&message.content) {
    for file_ref in refs {
      embeds.push(file_ref.create_embed().await);
    }
  }

  typing.stop().expect("Failed to stop typing");

  if !embeds.is_empty() {
    Some(embeds)
  } else {
    None
  }
}

async fn http_get_body_text(url: &String) -> Option<String> {
  match reqwest::get(url).await {
    Ok(res) => {
      match res.text().await {
        Ok(content) => Some(content),
        Err(e) => {
          println!("Failed to get text: {}", e);
          return None
        }
      }
    },
    Err(e) => {
      println!("Failed to get response: {}", e);
      return None
    }
  }
}

fn try_parse<F: FromStr + Clone>(m: Option<Match>) -> Option<F> {
  if let Some(m) = m {
    if let Ok(f) = m.as_str().parse::<F>() {
      return Some(f.to_owned())
    }
  }

  None
}

struct FileReference<'a> {
  owner: &'a str,
  repo: &'a str,
  git_ref: &'a str,
  path: &'a str,
  start: usize,
  end: Option<usize>
}

impl FileReference<'_> {
  pub fn try_from_str(text: &str) -> Option<Vec<FileReference>> {
    let r = Regex::new(r"https://github.com/(.+?)/(.+?)/blob/(.+?)/(.+?)#L([0-9]+)(?:-L([0-9]+))?")
      .expect("Expected url regex");

    let files: Vec<FileReference> = r.captures_iter(text)
      .map(|capture| {
        FileReference {
          owner: capture.get(1).expect("Expected owner").as_str(),
          repo: capture.get(2).expect("Expected repo").as_str(),
          git_ref: capture.get(3).expect("Expected git ref").as_str(),
          path: capture.get(4).expect("Expected file path").as_str(),
          start: try_parse::<usize>(capture.get(5)).expect("Expected start line"),
          end: try_parse::<usize>(capture.get(6))
        }
      })
      .collect();

    if !files.is_empty() {
      Some(files)
    } else {
      None
    }
  }

  pub async fn create_embed(&self) -> CreateEmbed {
    let extension = self.get_extension();

    let mut content = self.display().await.expect("Failed to get content");
    content.shrink_to(4096 - 8 - extension.len());

    let description = format!("```{}\n{}\n```", extension, content);

    let mut default = CreateEmbed::default();
    default.title(self.path)
      .description(description)
      .colour(ACCENT_COLOUR);

    default
  }

  fn get_extension(&self) -> String {
    Path::new(self.path)
      .extension()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string()
  }

  pub async fn display(&self) -> Option<String> {
    let url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", self.owner, self.repo, self.git_ref, self.path);
    println!("Downloading content: {}", url);

    if let Some(content) = http_get_body_text(&url).await {
      let lines: Vec<&str> = content.split("\n").collect();

      if let Some(end) = self.end {
        // return Some(lines.join("\n"))
        return Some(trim_indent(&lines[self.start..=end]))
        // return Some(trim_indent(&lines[self.start..=end]));
      } else {
        return Some(lines[self.start].trim_start().to_string())
      }
    } else {
      None
    }
  }
}
