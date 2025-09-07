pub(crate) const LAYOUT_SIZE: egui::Vec2 = egui::vec2(40.0, 40.0);

pub(crate) fn render_spinner(ui: &mut egui::Ui, angle: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let radius = 12.0;

    // spinning circle
    let stroke = egui::Stroke::new(3.0, egui::Color32::from_rgb(100, 150, 255));
    painter.circle_stroke(center, radius, stroke);
    // moving dot
    let dot_pos = center + egui::vec2(radius * angle.cos(), radius * angle.sin());
    painter.circle_filled(dot_pos, 4.0, egui::Color32::from_rgb(100, 150, 255));
}
