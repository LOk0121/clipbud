use eframe::egui;
use mouse_position::mouse_position::Mouse;
use std::sync::mpsc;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::clipboard;

use crate::ai::Config;

pub(crate) struct UI {
    clipboard_text: String,
    window_visible: bool,
    window_size: egui::Vec2,
    window_position: egui::Pos2,
    config: Config,

    // Loading state for actions
    is_loading: bool,
    loading_start_time: std::time::Instant,
    current_action_label: String,

    clipboard_rx: mpsc::Receiver<clipboard::Event>,
    action_response_rx: mpsc::Receiver<String>,
    action_response_tx: mpsc::Sender<String>,
    _tray_icon: TrayIcon,
    quit_menu_item: MenuItem,
}

impl UI {
    fn new(clipboard_rx: mpsc::Receiver<clipboard::Event>, config: Config) -> anyhow::Result<Self> {
        let (action_response_tx, action_response_rx) = mpsc::channel();

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

        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Clipboard Buddy")
            .with_title("📋")
            .build()?;

        Ok(Self {
            clipboard_text: String::new(),
            window_visible: false,
            window_size: egui::vec2(400.0, 200.0),
            window_position: egui::pos2(-2000.0, -2000.0),
            config,
            is_loading: false,
            loading_start_time: std::time::Instant::now(),
            current_action_label: String::new(),
            clipboard_rx,
            action_response_rx,
            action_response_tx,
            quit_menu_item,
            _tray_icon,
        })
    }

    fn show_on_clibboard_change(&mut self, ctx: &egui::Context) {
        if let Ok(event) = self.clipboard_rx.try_recv() {
            if event.text != self.clipboard_text {
                self.clipboard_text = event.text;
                self.show_window(ctx, event.mouse_x, event.mouse_y);
            }
        }
    }

    fn update_on_action_response(&mut self, _ctx: &egui::Context) {
        if let Ok(response) = self.action_response_rx.try_recv() {
            self.clipboard_text = response.clone();
            if let Err(e) = clipboard::set_clipboard_text(response) {
                eprintln!("failed to set clipboard text: {}", e);
            }
            // Stop loading when response is received
            self.is_loading = false;
            self.current_action_label.clear();
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

        // store the window position for mouse boundary checking
        self.window_position = egui::pos2(window_x, window_y);

        // position and resize the window at mouse cursor (with small offset)
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_position));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.window_size));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        self.window_visible = true;
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        // move window off-screen when hidden
        self.window_visible = false;
        self.window_position = egui::pos2(-2000.0, -2000.0);
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_position));
        ctx.request_repaint();
    }

    fn hide_if_esc_pressed(&mut self, ctx: &egui::Context) {
        // handle escape key to hide window
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide_window(ctx);
        }
    }

    fn hide_if_mouse_outside_window(&mut self, ctx: &egui::Context) {
        if !self.window_visible {
            return;
        }

        if let Mouse::Position { x, y } = Mouse::get_mouse_position() {
            let mouse_x = x as f32;
            let mouse_y = y as f32;

            // calculate window bounds
            let window_rect = egui::Rect::from_min_size(self.window_position, self.window_size);

            // add some margin to prevent accidental hiding when moving cursor to edges
            let margin = 20.0;
            let expanded_rect = window_rect.expand(margin);

            // check if mouse is outside the expanded window bounds
            let mouse_pos = egui::pos2(mouse_x, mouse_y);
            if !expanded_rect.contains(mouse_pos) {
                self.hide_window(ctx);
            }
        }
    }

    fn trigger_action_on_keypress(&mut self, ctx: &egui::Context) {
        if self.is_loading {
            return;
        }

        // check for action key presses
        for action in self.config.actions.iter() {
            if action.key.is_some()
                && ctx.input(|i| {
                    i.key_pressed(egui::Key::from_name(action.key.as_ref().unwrap()).unwrap())
                })
            {
                self.is_loading = true;
                self.loading_start_time = std::time::Instant::now();
                self.current_action_label = action.label.clone();
                action.trigger(&self.clipboard_text, self.action_response_tx.clone());
                break;
            }
        }
    }

    fn render_spinner(&self, ui: &mut egui::Ui) {
        let elapsed = self.loading_start_time.elapsed().as_secs_f32();
        let radians_per_second = elapsed * 2.0;

        ui.allocate_ui_with_layout(
            egui::vec2(40.0, 40.0),
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                let painter = ui.painter();
                let center = rect.center();
                let radius = 12.0;

                // spinning circle
                let stroke = egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 150, 255));
                painter.circle_stroke(center, radius, stroke);
                // moving dot
                let dot_angle = radians_per_second;
                let dot_pos =
                    center + egui::vec2(radius * dot_angle.cos(), radius * dot_angle.sin());
                painter.circle_filled(dot_pos, 4.0, egui::Color32::from_rgb(100, 150, 255));
            },
        );
    }

    fn render(&mut self, ctx: &egui::Context) {
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

                    if self.is_loading {
                        // Show loading state with current action label
                        ui.horizontal(|ui| {
                            self.render_spinner(ui);
                            ui.label(&self.current_action_label);
                        });
                        // Buttons are hidden during loading (no buttons shown at all)
                    } else {
                        // Normal state - buttons are interactive
                        for action in self.config.actions.iter() {
                            if ui.button(action.button_text()).clicked() {
                                self.is_loading = true;
                                self.loading_start_time = std::time::Instant::now();
                                self.current_action_label = action.label.clone();
                                action
                                    .trigger(&self.clipboard_text, self.action_response_tx.clone());
                            }
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
        self.show_on_clibboard_change(ctx);
        self.hide_if_esc_pressed(ctx);
        self.trigger_action_on_keypress(ctx);
        self.update_on_action_response(ctx);
        self.hide_if_mouse_outside_window(ctx);

        // always request repaint to ensure we process channel messages
        ctx.request_repaint();

        self.render(ctx);
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
