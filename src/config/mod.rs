use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

mod action;

pub(crate) use action::Action;
pub(crate) use action::Event as ActionEvent;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub theme: Option<String>,
    pub hotkey: Option<String>,
    pub actions: Vec<Action>,
    #[serde(default = "HashMap::new")]
    pub keys: HashMap<String, String>,
}

impl Config {
    pub fn default_path() -> PathBuf {
        PathBuf::from(shellexpand::full("~/.clipbud/").unwrap().to_string())
    }

    pub fn default_lock_file() -> PathBuf {
        Self::default_path().join(".lock")
    }

    pub fn compile(&mut self) -> anyhow::Result<()> {
        // set environment variables if defined in the config file
        for (key, value) in self.keys.iter() {
            println!("setting environment variable {}", key);
            unsafe {
                std::env::set_var(key, value);
            }
        }

        for action in self.actions.iter_mut() {
            action.compile()?;
        }
        Ok(())
    }

    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let path = shellexpand::full(path)?.to_string();
        println!("loading config from: {}", path);
        let config = std::fs::read_to_string(path)?;
        let mut config = serde_yaml::from_str::<Self>(&config)?;
        config.compile()?;
        Ok(config)
    }
}
