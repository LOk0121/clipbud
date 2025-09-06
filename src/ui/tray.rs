use std::io::Cursor;

use image::ImageReader;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
};

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

pub(crate) fn build_tray_menu_icon() -> anyhow::Result<(TrayIcon, MenuItem)> {
    let tray_menu = Menu::new();
    let quit_menu_item = MenuItem::new("Quit", true, None);
    let (icon, menu_icon) = load_icons();

    tray_menu.append_items(&[
        &MenuItem::new("Clipboard Buddy", false, None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("Clipboard Buddy".to_string()),
                icon: Some(menu_icon),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                copyright: Some("Copyright Simone 'evilsocket' Margaritelli @ 2025".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_menu_item,
    ])?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Clipboard Buddy")
        // .with_title("📋")
        .with_icon(icon)
        .build()?;

    Ok((tray_icon, quit_menu_item))
}
