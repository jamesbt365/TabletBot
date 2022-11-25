use serde_json::{from_reader, to_writer_pretty};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions };
use std::path::Path;
use super::Snippet;

#[derive(Deserialize, Serialize)]
pub struct State {
  pub snippets: Vec<Snippet>
}

impl Default for State {
  fn default() -> State {
    Self {
      snippets: Vec::new()
    }
  }
}

impl State {
  pub fn read(file_path: &str) -> State {
    let path = Path::new(file_path);

    if path.exists() {
      let file = File::open(file_path).unwrap();
      from_reader(file).unwrap()
    } else {
      State::default()
    }
  }

  pub fn write(&self, file_path: &str) {
    let path = Path::new(file_path);

    if path.exists() {
      fs::remove_file(path).expect("Failed to delete old file.");
    }

    let result = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(path);

    match result {
      Ok(file) => to_writer_pretty(file, self).expect("Failed to write"),
      Err(e) => println!("Unable to write state to {}: {}", file_path, e)
    };
  }
}
