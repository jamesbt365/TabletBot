use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer_pretty};
use serenity::builder::CreateEmbed;
use serenity::prelude::TypeMapKey;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::Path;

pub trait Embeddable {
    fn embed(&self) -> CreateEmbed;
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub content: String,
}

impl Snippet {
    pub fn format_output(&self) -> String {
        format!("{}: {}", self.id, self.title)
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct SnippetState {
    pub snippets: Vec<Snippet>,
}

impl TypeMapKey for SnippetState {
    type Value = SnippetState;
}

impl SnippetState {
    pub fn get_path() -> String {
        let pwd = env::current_dir().unwrap().to_string_lossy().to_string();
        let data_root = env::var("TABLETBOT_DATA").unwrap_or(pwd);

        match env::var("TABLETBOT_STATE") {
            Ok(path) => path,
            Err(_) => format!("{data_root}/state.json"),
        }
    }

    pub fn read() -> SnippetState {
        let path_str = Self::get_path();
        let path = Path::new(&path_str);

        if path.exists() {
            let file = File::open(path).unwrap();
            from_reader(file).unwrap()
        } else {
            SnippetState::default()
        }
    }

    pub fn write(&self) {
        let path_str = Self::get_path();
        let path = Path::new(&path_str);

        if path.exists() {
            fs::remove_file(path).expect("Failed to delete old file.");
        }

        let writer = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path);

        match writer {
            Ok(writer) => match to_writer_pretty(writer, self) {
                Ok(_) => println!("Successfully saved state to '{path_str}'",),
                Err(e) => println!("Failed to save state to '{path_str}': {e}"),
            },
            Err(e) => println!("Unable to write state to '{path_str}': {e}"),
        };
    }
}
