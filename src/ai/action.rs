use std::sync::mpsc;

use rig::{
    agent::Agent,
    client::{builder::DynClientBuilder, completion::CompletionModelHandle},
    completion::Prompt,
    providers::gemini::completion::gemini_api_types::{AdditionalParameters, GenerationConfig},
};
use serde::Deserialize;

pub(crate) enum Event {
    Response(String, bool),
    Error(anyhow::Error),
}

#[derive(Deserialize)]
pub(crate) struct Action {
    pub label: String,
    pub prompt: String,
    pub key: Option<String>,
    pub model: String,
    pub provider: String,
    #[serde(default = "default_paste")]
    pub paste: bool,

    #[serde(skip)]
    agent: Option<Agent<CompletionModelHandle<'static>>>,
}

fn default_paste() -> bool {
    true
}

impl Action {
    pub fn compile(&mut self) -> anyhow::Result<()> {
        let mut builder = DynClientBuilder::new()
            .agent(&self.provider, &self.model)?
            .preamble(include_str!("system.md"));

        // handle google provider
        if self.provider == "google" {
            self.provider = "gemini".to_string();
        }

        if self.provider == "gemini" {
            // if not passed gemini will return an error
            let gen_config = GenerationConfig::default();
            let config =
                serde_json::to_value(AdditionalParameters::default().with_config(gen_config))?;

            println!("gen_config: {:?}", config);

            builder = builder.additional_params(config);
        }

        self.agent = Some(builder.build());
        Ok(())
    }

    pub fn button_text(&self) -> String {
        if self.key.is_none() {
            self.label.clone()
        } else {
            format!("[{}] {}", self.key.as_ref().unwrap(), self.label)
        }
    }

    pub fn trigger(&self, clipboard_text: &str, action_response_tx: mpsc::Sender<Event>) {
        if let Some(agent) = &self.agent {
            let prompt = self.prompt.clone() + "\n\n" + clipboard_text;
            let agent = agent.clone();
            let do_paste = self.paste;

            std::thread::spawn(move || {
                // ugly hack to call async code from a sync context
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build();

                if let Err(e) = runtime {
                    action_response_tx
                        .send(Event::Error(anyhow::anyhow!("{}", e)))
                        .unwrap();
                    return;
                }

                match runtime
                    .unwrap()
                    .block_on(async { agent.prompt(prompt).await })
                {
                    Ok(response) => action_response_tx
                        .send(Event::Response(response, do_paste))
                        .unwrap(),
                    Err(e) => action_response_tx
                        .send(Event::Error(anyhow::anyhow!("{}", e)))
                        .unwrap(),
                }
            });
        } else {
            action_response_tx
                .send(Event::Error(anyhow::anyhow!("action not compiled")))
                .unwrap();
        }
    }
}
