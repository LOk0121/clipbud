use std::io::Cursor;

use image::ImageReader;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
};

use crate::ai::Config;

pub(crate) fn open_config_folder() -> anyhow::Result<()> {
    let config_path = Config::default_path();

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_path)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(&config_path)
            .spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_path)
            .spawn()?;
    }

    Ok(())
}

pub(crate) fn reload_config() -> anyhow::Result<()> {
    // restart the process to reload configuration
    let current_exe = std::env::current_exe()?;
    let mut command = std::process::Command::new(current_exe);
    // add all original command line arguments if any
    let args: Vec<String> = std::env::args().skip(1).collect();
    command.args(&args);
    // add the start delay argument to ensure the single instance is released
    if !args.contains(&"--start-delay".to_string()) {
        command.arg("--start-delay").arg("500");
    }
    // go go go!
    command.spawn()?;

    std::process::exit(0);

    #[allow(unreachable_code)]
    Ok(())
}

fn load_icons() -> (tray_icon::Icon, tray_icon::menu::Icon) {
    let (icon_rgba, icon_width, icon_height) = {
        let bytes = include_bytes!("../../assets/icon-256.png");
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap()
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    (
        tray_icon::Icon::from_rgba(icon_rgba.clone(), icon_width, icon_height).unwrap(),
        tray_icon::menu::Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap(),
    )
}

pub(crate) struct Tray {
    _icon: TrayIcon,
    pub configure_menu_item: MenuItem,
    pub reload_menu_item: MenuItem,
    pub quit_menu_item: MenuItem,
}

pub(crate) fn build_tray_menu_icon() -> anyhow::Result<Tray> {
    let tray_menu = Menu::new();
    let configure_menu_item = MenuItem::new("Configure", true, None);
    let reload_menu_item = MenuItem::new("Reload Configuration", true, None);
    let quit_menu_item = MenuItem::new("Quit", true, None);
    let (icon, menu_icon) = load_icons();

    tray_menu.append_items(&[
        &MenuItem::new("Clipboard Buddy", false, None),
        &PredefinedMenuItem::separator(),
        &configure_menu_item,
        &reload_menu_item,
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("Clipboard Buddy".to_string()),
                icon: Some(menu_icon),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                copyright: Some("Copyright Simone 'evilsocket' Margaritelli @ 2025".to_string()),
                website: Some("https://github.com/evilsocket/clipbud".to_string()),
                ..Default::default()
            }),
        ),
        &quit_menu_item,
    ])?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Clipboard Buddy")
        // .with_title("📋")
        .with_icon(icon)
        .build()?;

    Ok(Tray {
        _icon: tray_icon,
        configure_menu_item,
        reload_menu_item,
        quit_menu_item,
    })
}
