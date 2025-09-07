use eframe::egui;

pub fn show_error(message: String) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 150.0])
            .with_resizable(false)
            .with_always_on_top()
            .with_transparent(false),
        ..Default::default()
    };

    eframe::run_simple_native("Clipboard Buddy - Error", options, move |ctx, _frame| {
        let mut should_close = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add margin around the content
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(&message)
                        .size(15.0)
                        .color(egui::Color32::RED),
                );
                ui.add_space(50.0);

                if ui
                    .button(egui::RichText::new(" Close ").size(15.0))
                    .clicked()
                {
                    should_close = true;
                }
            });
        });

        if should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    })
    .unwrap();
}
