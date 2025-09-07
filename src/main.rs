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
    #[arg(long)]
    start_delay: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    // if we were restarted, wait a bit before restarting to ensure single instance is released
    if let Some(start_delay) = args.start_delay {
        std::thread::sleep(std::time::Duration::from_millis(start_delay));
    }

    // create user data if needed
    if let Err(e) = ai::Config::create_user_data() {
        ui::dialogs::show_error(format!("Could not create user data: {}", e));
        return Ok(());
    }

    // make sure we're the only instance running
    if !SingleInstance::new(Config::default_lock_file().to_str().unwrap())?.is_single() {
        ui::dialogs::show_error("Clipboard Buddy is already running.".to_string());
        return Ok(());
    }

    // load config
    let config = match ai::Config::from_file(
        &args.config.unwrap_or(
            ai::Config::default_config_file()
                .to_str()
                .unwrap()
                .to_string(),
        ),
    ) {
        Ok(config) => config,
        Err(e) => {
            ui::dialogs::show_error(format!("Could not load configuration: {}", e));
            return Ok(());
        }
    };

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
    if let Err(e) = ui::run(event_rx, config) {
        ui::dialogs::show_error(format!("Could not run UI: {}", e));
    }

    Ok(())
}
