use std::env;
use std::sync::Arc;
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::{TypeMapKey, Mutex};
use super::State;

pub struct Container {
  pub state: State,
  state_path: String
}

impl Container {
  pub fn new(data_root: &str) -> Container {
    let state_path = match env::var("TABLETBOT_STATE") {
      Ok(path) => path,
      Err(_) => format!("{}/state.json", data_root)
    };

    Container {
      state: State::read(&state_path),
      state_path
    }
  }

  pub fn write(&self) {
    self.state.write(&self.state_path);
  }
}

impl TypeMapKey for Container {
  type Value = Container;
}

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}
