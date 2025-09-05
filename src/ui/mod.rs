use eframe::egui;
use std::sync::mpsc;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::clipboard;

mod config;

pub(crate) use config::{Action, Config};

pub(crate) struct UI {
    clipboard_text: String,
    window_visible: bool,
    window_size: egui::Vec2,
    config: Config,

    clipboard_rx: mpsc::Receiver<clipboard::Event>,
    action_response_rx: mpsc::Receiver<String>,
    action_response_tx: mpsc::Sender<String>,
    tray_icon: TrayIcon,
    quit_menu_item: MenuItem,
}

impl UI {
    fn new(clipboard_rx: mpsc::Receiver<clipboard::Event>, config: Config) -> anyhow::Result<Self> {
        let tray_menu = Menu::new();

        let quit_menu_item = MenuItem::new("Quit", true, None);
        tray_menu.append_items(&[
            &MenuItem::new("Clipboard Buddy", false, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("Clipboard Buddy".to_string()),
                    copyright: Some(
                        "Copyright Simone 'evilsocket' Margaritelli @ 2025".to_string(),
                    ),
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

        let (action_response_tx, action_response_rx) = mpsc::channel();
        Ok(Self {
            clipboard_text: String::new(),
            window_visible: false,
            window_size: egui::vec2(400.0, 200.0),
            clipboard_rx,
            action_response_rx,
            action_response_tx,
            config,
            quit_menu_item,
            tray_icon,
        })
    }

    fn handle_clipboard_data_change(&mut self, ctx: &egui::Context) {
        if let Ok(event) = self.clipboard_rx.try_recv() {
            if event.text != self.clipboard_text {
                self.clipboard_text = event.text;
                self.show_window(ctx, event.mouse_x, event.mouse_y);
            }
        }
    }

    fn handle_action_response(&mut self, ctx: &egui::Context) {
        if let Ok(response) = self.action_response_rx.try_recv() {
            self.clipboard_text = response.clone();
            if let Err(e) = clipboard::set_clipboard_text(response) {
                eprintln!("failed to set clipboard text: {}", e);
            }
            // self.hide_window(ctx);
        }
    }

    fn handle_menu_event(&mut self) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_menu_item.id() {
                std::process::exit(0)
            }
        }
    }

    fn show_window(&mut self, ctx: &egui::Context, mouse_x: f32, mouse_y: f32) {
        // ensure window stays within screen bounds
        // let screen_rect = ctx.screen_rect();
        let offset = 10.0;
        let window_x = mouse_x + offset;
        let window_y = mouse_y + offset;

        /*
        // adjust x position if window would go off right edge
        if window_x + self.window_size.x > screen_rect.max.x {
            window_x = screen_rect.max.x - self.window_size.x - offset;
        }
        // adjust x position if window would go off left edge
        if window_x < screen_rect.min.x {
            window_x = screen_rect.min.x + offset;
        }

        // adjust y position if window would go off bottom edge
        if window_y + self.window_size.y > screen_rect.max.y {
            window_y = screen_rect.max.y - self.window_size.y - offset;
        }
        // adjust y position if window would go off top edge
        if window_y < screen_rect.min.y {
            window_y = screen_rect.min.y + offset;
        }
         */

        // position and resize the window at mouse cursor (with small offset)
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
            window_x, window_y,
        )));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.window_size));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        self.window_visible = true;
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        // move window off-screen when hidden
        self.window_visible = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
            -2000.0, -2000.0,
        )));
    }

    fn handle_esc_key(&mut self, ctx: &egui::Context) {
        // handle escape key to hide window
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide_window(ctx);
        }
    }

    fn handle_action_key(&mut self, ctx: &egui::Context) {
        // check for action key presses
        for action in self.config.actions.iter() {
            if ctx.input(|i| i.key_pressed(egui::Key::from_name(&action.key).unwrap())) {
                action.trigger(&self.clipboard_text, self.action_response_tx.clone());
                break;
            }
        }
    }

    fn render_window(&mut self, ctx: &egui::Context) {
        if self.window_visible {
            egui::CentralPanel::default().show(ctx, |ui| {
                let style = ui.style_mut();
                style.visuals.window_fill = egui::Color32::from_rgb(240, 240, 240);
                style.spacing.item_spacing = egui::vec2(8.0, 4.0);
                style.spacing.window_margin = egui::Margin::same(12.0);

                ui.vertical(|ui| {
                    ui.label("📋 Clipboard Buddy");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(70.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.clipboard_text.as_str())
                                    .desired_width(350.0)
                                    .interactive(false)
                                    .font(egui::TextStyle::Monospace),
                            );
                        });

                    ui.separator();

                    for action in self.config.actions.iter() {
                        if ui
                            .button(format!("[{}] {}", action.key, action.label))
                            .clicked()
                        {
                            action.trigger(&self.clipboard_text, self.action_response_tx.clone());
                        }
                    }

                    ui.separator();
                    ui.small("[ESC] Hide");
                });
            });
        }
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_menu_event();
        self.handle_clipboard_data_change(ctx);
        self.handle_esc_key(ctx);
        self.handle_action_key(ctx);
        self.handle_action_response(ctx);

        // always request repaint to ensure we process channel messages
        ctx.request_repaint();

        self.render_window(ctx);
    }
}

pub fn run(event_rx: mpsc::Receiver<clipboard::Event>, config: Config) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Clipboard Buddy")
            .with_inner_size([400.0, 200.0])
            .with_position([-2000.0, -2000.0]) // Start off-screen
            .with_decorations(false) // Borderless window
            .with_transparent(false) // Disable transparency for better visibility
            .with_always_on_top() // Always on top
            .with_resizable(false) // Fixed size
            .with_visible(true), // Must be visible (we control visibility via position)
        ..Default::default()
    };

    let ui = UI::new(event_rx, config)?;
    if let Err(e) = eframe::run_native("Clipboard Buddy", options, Box::new(|_cc| Ok(Box::new(ui))))
    {
        Err(anyhow::anyhow!("eframe::run_native error: {}", e))
    } else {
        Ok(())
    }
}
