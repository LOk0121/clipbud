use eframe::egui;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use mouse_position::mouse_position::Mouse;
use std::{str::FromStr, sync::mpsc};
use tray_icon::{
    TrayIcon,
    menu::{MenuEvent, MenuItem},
};

use crate::config::Config;
use crate::{clipboard, config};

mod tray;
pub(crate) struct UI {
    config: Config,

    window_visible: bool,
    window_size: egui::Vec2,
    window_position: egui::Pos2,

    clipboard_text: Option<String>,

    is_loading: bool,
    loading_start_time: std::time::Instant,
    current_action_label: String,

    // modal state
    show_error_modal: bool,
    error_message: String,

    clipboard_rx: mpsc::Receiver<clipboard::Event>,
    action_response_rx: mpsc::Receiver<config::ActionEvent>,
    action_response_tx: mpsc::Sender<config::ActionEvent>,
    _tray_icon: TrayIcon,
    configure_menu_item: MenuItem,
    quit_menu_item: MenuItem,
    _hotkey_manager: Option<GlobalHotKeyManager>,
}

impl UI {
    fn new(
        creation_context: &eframe::CreationContext<'_>,
        clipboard_rx: mpsc::Receiver<clipboard::Event>,
        config: Config,
    ) -> anyhow::Result<Self> {
        let (action_response_tx, action_response_rx) = mpsc::channel();

        let (_tray_icon, configure_menu_item, quit_menu_item) = tray::build_tray_menu_icon()?;

        let mut _hotkey_manager = None;
        if let Some(hotkey) = config.hotkey.as_ref() {
            let manager = GlobalHotKeyManager::new()?;
            let hotkey = HotKey::from_str(hotkey)?;
            println!("registering for hotkey: {}", hotkey);
            manager.register(hotkey)?;
            _hotkey_manager = Some(manager);
        } else {
            println!("registering for clipboard change")
        }

        if let Some(theme) = config.theme.as_ref() {
            match theme.as_str() {
                "dark" => creation_context.egui_ctx.set_theme(egui::Theme::Dark),
                "light" => creation_context.egui_ctx.set_theme(egui::Theme::Light),
                "system" => {}
                _ => return Err(anyhow::anyhow!("invalid theme: {}", theme)),
            }
        }

        Ok(Self {
            clipboard_text: None,
            window_visible: false,
            window_size: egui::vec2(400.0, 200.0),
            window_position: egui::pos2(-2000.0, -2000.0),
            config,
            is_loading: false,
            loading_start_time: std::time::Instant::now(),
            current_action_label: String::new(),
            show_error_modal: false,
            error_message: String::new(),
            clipboard_rx,
            action_response_rx,
            action_response_tx,
            configure_menu_item,
            quit_menu_item,
            _tray_icon,
            _hotkey_manager,
        })
    }

    fn show_on_clibboard_change_or_hotkey(&mut self, ctx: &egui::Context) {
        let mut do_show = false;

        // update clipboard text
        if let Ok(event) = self.clipboard_rx.try_recv() {
            self.clipboard_text = Some(event.text.clone());
            // if no hotkey is set, show window
            if self.config.hotkey.is_none() {
                do_show = true;
            }
        }

        // check for hotkey press if configured
        if self.config.hotkey.is_some()
            && let Ok(_) = GlobalHotKeyEvent::receiver().try_recv()
        {
            do_show = true;
        }

        // show window if needed at the last clipboard change mouse position
        if do_show {
            // get current mouse position
            let (x, y) = match Mouse::get_mouse_position() {
                Mouse::Position { x, y } => (x as f32, y as f32),
                _ => (0.0, 0.0),
            };
            self.show_window(ctx, x, y);
        }
    }

    fn update_on_action_response(&mut self, _ctx: &egui::Context) {
        if let Ok(response) = self.action_response_rx.try_recv() {
            // stop loading when response is received
            self.is_loading = false;
            self.current_action_label.clear();

            match response {
                config::ActionEvent::Response(response) => {
                    self.clipboard_text = Some(response.clone());
                    if let Err(e) = clipboard::set_clipboard_text(response) {
                        eprintln!("failed to set clipboard text: {}", e);
                    }
                }
                config::ActionEvent::Error(error) => {
                    self.error_message = format!("❌ {}", error);
                    self.show_error_modal = true;
                }
            }
        }
    }

    fn handle_menu_event(&mut self) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_menu_item.id() {
                std::process::exit(0)
            } else if event.id == self.configure_menu_item.id()
                && let Err(e) = tray::open_config_folder()
            {
                self.error_message = format!("❌ Failed to open config folder: {}", e);
                self.show_error_modal = true;
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

        TODO: add screen bounds checking

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
        if !self.is_loading {
            // move window off-screen when hidden
            self.window_visible = false;
            self.window_position = egui::pos2(-2000.0, -2000.0);
            self.show_error_modal = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_position));
            ctx.request_repaint();
        }
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

    fn show_error_modal(&mut self, ctx: &egui::Context) {
        if self.show_error_modal {
            let mut should_close = false;
            let mut show_modal = self.show_error_modal;

            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut show_modal)
                .show(ctx, |ui| {
                    ui.label(self.error_message.clone());
                    ui.add_space(10.0);
                    if ui.button("OK").clicked() {
                        should_close = true;
                    }
                });

            if should_close || !show_modal {
                self.show_error_modal = false;
                self.error_message.clear();
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
                if let Some(clipboard_text) = self.clipboard_text.as_ref() {
                    self.is_loading = true;
                    self.loading_start_time = std::time::Instant::now();
                    self.current_action_label = action.label.clone();
                    action.trigger(clipboard_text, self.action_response_tx.clone());
                } else {
                    self.show_error_modal = true;
                    self.error_message = "❌ No clipboard text found".to_string();
                }
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
                    ui.label(format!("📋 Clipboard Buddy v{}", env!("CARGO_PKG_VERSION")));
                    ui.separator();

                    let mut clipboard_text = if let Some(text) = self.clipboard_text.as_ref() {
                        text.clone()
                    } else {
                        "❌ No clipboard text found.".to_string()
                    };

                    egui::ScrollArea::vertical()
                        .max_height(70.0)
                        .show(ui, |ui| {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut clipboard_text)
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
                        ui.horizontal(|ui| {
                            ui.columns(self.config.actions.len(), |columns| {
                                for (i, action) in self.config.actions.iter().enumerate() {
                                    if columns[i].small_button(action.button_text()).clicked() {
                                        if let Some(clipboard_text) = self.clipboard_text.as_ref() {
                                            self.is_loading = true;
                                            self.loading_start_time = std::time::Instant::now();
                                            self.current_action_label = action.label.clone();
                                            action.trigger(
                                                clipboard_text,
                                                self.action_response_tx.clone(),
                                            );
                                        } else {
                                            self.show_error_modal = true;
                                            self.error_message =
                                                "❌ No clipboard text found".to_string();
                                        }
                                    }
                                }
                            });
                        });
                    }
                });
            });
        }
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_menu_event();
        self.show_on_clibboard_change_or_hotkey(ctx);
        self.hide_if_esc_pressed(ctx);
        self.trigger_action_on_keypress(ctx);
        self.update_on_action_response(ctx);
        self.hide_if_mouse_outside_window(ctx);
        self.show_error_modal(ctx);

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
            // start off-screen
            .with_position([-2000.0, -2000.0])
            // borderless window
            .with_decorations(false)
            // disable transparency for better visibility
            .with_transparent(false)
            // always on top
            .with_always_on_top()
            // fixed size
            .with_resizable(false)
            // must be visible (we control visibility via position)
            .with_visible(true)
            // add icon
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Clipboard Buddy",
        options,
        Box::new(|cc| Ok(Box::new(UI::new(cc, event_rx, config)?))),
    ) {
        Err(anyhow::anyhow!("eframe::run_native error: {}", e))
    } else {
        Ok(())
    }
}
