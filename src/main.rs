use std::sync::mpsc;

mod clipboard;
mod ui;

fn main() -> anyhow::Result<()> {
    let config = ui::Config {
        actions: vec![
            ui::Action::new(
                "Fix typos and grammar".to_string(),
                "Fix typos and grammar of the following text, only return the fixed text and nothing else:".to_string(),
                "T".to_string(),
                "gpt-4o".to_string(),
                "openai".to_string(),
            )?,
            ui::Action::new(
                "Summarize".to_string(),
                "Summarize the following text, only return the summary and nothing else:".to_string(),
                "S".to_string(),
                "gpt-4o".to_string(),
                 "openai".to_string(),
            )?,
            ui::Action::new(
                "Make Friendly".to_string(),
                "Make the following text more friendly, only return the friendly text and nothing else:".to_string(),
                "F".to_string(),
                "gpt-4o".to_string(),
                "openai".to_string(),
            )?,
        ],
    };
    let (event_tx, event_rx) = mpsc::channel();

    let shutdown = clipboard::start_observer(event_tx);
    ctrlc::set_handler(move || {
        if let Some(shutdown) = shutdown.lock().unwrap().take() {
            println!("stopping clipboard watcher...");
            shutdown.stop();
        }
        println!("exiting application...");
        std::process::exit(0);
    })
    .expect("error setting Ctrl-C handler");

    ui::run(event_rx, config)
}
