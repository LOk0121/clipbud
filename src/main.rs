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
    }

    let default_config = config::Config::default_path().join("config.yml");
    if !default_config.exists() {
        println!(
            "creating default config file at {}",
            default_config.to_str().unwrap()
        );
        let default_data = include_str!("config/default.yml");
        std::fs::write(&default_config, default_data)?;
    }

    // make sure we're the only instance running
    let instance = SingleInstance::new(Config::default_lock_file().to_str().unwrap())?;
    if !instance.is_single() {
        println!("clipbud is already running, exiting...");
        return Ok(());
    }

    // load config
    let args = Arguments::parse();
    let config = config::Config::from_file(
        &args
            .config
            .unwrap_or(default_config.to_str().unwrap().to_string()),
    )?;

    // set environment variables if defined in the config file
    for (key, value) in config.keys.iter() {
        println!("setting environment variable {}", key);
        unsafe {
            std::env::set_var(key, value);
        }
    }

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
