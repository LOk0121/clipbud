#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::sync::mpsc;

use clap::Parser;
use single_instance::SingleInstance;

use crate::config::Config;

mod clipboard;
mod config;
mod ui;

#[derive(Debug, Parser)]
struct Arguments {
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let user_path = config::Config::default_path();
    if !user_path.exists() {
        std::fs::create_dir_all(&user_path)?;
        // TODO: copy default config file
    }

    let instance = SingleInstance::new(Config::default_lock_file().to_str().unwrap())?;
    if !instance.is_single() {
        println!("clipbud is already running, exiting...");
        return Ok(());
    }

    let args = Arguments::parse();
    let config = config::Config::from_file(
        &args
            .config
            .unwrap_or(user_path.join("config.yaml").to_str().unwrap().to_string()),
    )?;

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
