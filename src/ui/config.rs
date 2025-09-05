use std::sync::mpsc;

use rig::{
    agent::Agent,
    client::{builder::DynClientBuilder, completion::CompletionModelHandle},
    completion::Prompt,
};

pub(crate) struct Config {
    pub actions: Vec<Action>,
}

pub(crate) struct Action {
    pub label: String,
    pub prompt: String,
    pub key: String,

    agent: Agent<CompletionModelHandle<'static>>,
}

impl Action {
    pub fn new(
        label: String,
        prompt: String,
        key: String,
        model: String,
        provider: String,
    ) -> anyhow::Result<Self> {
        let agent = DynClientBuilder::new().agent(&provider, &model)?.build();
        Ok(Self {
            label,
            prompt,
            key,
            agent,
        })
    }

    pub fn trigger(&self, clipboard_text: &str, action_response_tx: mpsc::Sender<String>) {
        let prompt = self.prompt.clone() + "\n\n" + clipboard_text;
        let agent = self.agent.clone();
        std::thread::spawn(move || {
            let response = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { agent.prompt(prompt).await })
                .unwrap();

            action_response_tx.send(response).unwrap();
        });
    }
}
