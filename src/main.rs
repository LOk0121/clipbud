#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::sync::mpsc;

use clap::Parser;
use single_instance::SingleInstance;

use crate::ai::Config;

mod ai;
mod clipboard;
mod ui;

#[derive(Debug, Parser)]
struct Arguments {
    #[arg(short, long)]
    config: Option<String>,
    #[arg(short, long)]
    start_delay: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    // if we were restarted, wait a bit before restarting to ensure single instance is released
    if let Some(start_delay) = args.start_delay {
        std::thread::sleep(std::time::Duration::from_millis(start_delay));
    }

    // make sure we're the only instance running
    if !SingleInstance::new(Config::default_lock_file().to_str().unwrap())?.is_single() {
        println!("clipbud is already running, exiting...");
        return Ok(());
    }

    // load config
    let config = ai::Config::from_file(
        &args.config.unwrap_or(
            ai::Config::default_config_file()
                .to_str()
                .unwrap()
                .to_string(),
        ),
    )?;

    // prepare comms
    let (event_tx, event_rx) = mpsc::channel();

    // start clipboard observer
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

    // build and run the UI event loop
    ui::run(event_rx, config)
}
