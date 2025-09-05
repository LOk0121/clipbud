use std::sync::mpsc;

use clap::Parser;

mod ai;
mod clipboard;
mod ui;

#[derive(Debug, Parser)]
struct Arguments {
    #[arg(short, long, default_value = "~/.clipbud/config.yaml")]
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();
    let config = ai::Config::from_file(&args.config)?;

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
