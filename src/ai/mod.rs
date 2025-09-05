use std::sync::mpsc;

use rig::{
    agent::Agent,
    client::{builder::DynClientBuilder, completion::CompletionModelHandle},
    completion::Prompt,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub hotkey: Option<String>,
    pub actions: Vec<Action>,
}

impl Config {
    pub fn compile(&mut self) -> anyhow::Result<()> {
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

#[derive(Deserialize)]
pub(crate) struct Action {
    pub label: String,
    pub prompt: String,
    pub key: Option<String>,
    pub model: String,
    pub provider: String,

    #[serde(skip)]
    agent: Option<Agent<CompletionModelHandle<'static>>>,
}

impl Action {
    pub fn compile(&mut self) -> anyhow::Result<()> {
        let agent = DynClientBuilder::new()
            .agent(&self.provider, &self.model)?
            .build();
        self.agent = Some(agent);
        Ok(())
    }

    pub fn button_text(&self) -> String {
        if self.key.is_none() {
            self.label.clone()
        } else {
            format!("[{}] {}", self.key.as_ref().unwrap(), self.label)
        }
    }

    pub fn trigger(&self, clipboard_text: &str, action_response_tx: mpsc::Sender<String>) {
        if let Some(agent) = &self.agent {
            let prompt = self.prompt.clone() + "\n\n" + clipboard_text;
            let agent = agent.clone();
            std::thread::spawn(move || {
                // ugly hack to call async code from a sync context
                let response = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async { agent.prompt(prompt).await })
                    .unwrap();

                action_response_tx.send(response).unwrap();
            });
        } else {
            eprintln!("action not compiled");
        }
    }
}
