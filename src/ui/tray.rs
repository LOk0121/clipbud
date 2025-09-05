use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
};

pub(crate) fn build_tray_menu_icon() -> anyhow::Result<(TrayIcon, MenuItem)> {
    let tray_menu = Menu::new();
    let quit_menu_item = MenuItem::new("Quit", true, None);

    tray_menu.append_items(&[
        &MenuItem::new("Clipboard Buddy", false, None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("Clipboard Buddy".to_string()),
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
        .with_title("📋")
        .build()?;

    Ok((tray_icon, quit_menu_item))
}
