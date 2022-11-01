mod container;
mod state;

pub use container::*;
pub use state::*;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ReactionRole {
  pub message_id: u64,
  pub role_id: u64,
  pub emote: String
}

#[derive(Deserialize, Serialize)]
pub struct Snippet {
  pub id: String,
  pub title: String,
  pub content: String
}
