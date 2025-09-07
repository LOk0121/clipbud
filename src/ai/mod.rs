use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

mod action;

pub(crate) use action::Action;
pub(crate) use action::Event as ActionEvent;

#[derive(Deserialize)]
pub(crate) enum ButtonsWrap {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "none")]
    None,
}

#[derive(Deserialize)]
pub(crate) struct Config {
    pub theme: Option<String>,
    pub hotkey: Option<String>,
    #[serde(default = "default_buttons_wrap")]
    pub wrap_buttons: ButtonsWrap,
    pub actions: Vec<Action>,
    #[serde(default = "HashMap::new")]
    pub keys: HashMap<String, String>,
}

fn default_buttons_wrap() -> ButtonsWrap {
    ButtonsWrap::Horizontal
}

impl Config {
    pub fn default_path() -> PathBuf {
        PathBuf::from(shellexpand::full("~/.clipbud/").unwrap().to_string())
    }

    pub fn default_config_file() -> PathBuf {
        Self::default_path().join("config.yml")
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
        // create the user folder if needed
        let user_path = Self::default_path();
        if !user_path.exists() {
            std::fs::create_dir_all(&user_path)?;
        }

        // install the default config file
        let default_config = Self::default_config_file();
        if !default_config.exists() {
            println!(
                "creating default config file at {}",
                default_config.to_str().unwrap()
            );
            let default_data = include_str!("default-config.yml");
            std::fs::write(&default_config, default_data)?;
        }

        let path = shellexpand::full(path)?.to_string();
        println!("loading config from: {}", path);
        let config = std::fs::read_to_string(path)?;
        let mut config = serde_yaml::from_str::<Self>(&config)?;
        config.compile()?;
        Ok(config)
    }
}
