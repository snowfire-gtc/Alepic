use eframe::egui;
use crate::canvas::palette::Palette;

pub fn render_color_layer(ctx: &egui::Context, selected_color: &mut u8, on_save: &mut dyn FnMut()) {
    // 1. Палитра (Bottom Center/Left)
    egui::Area::new("color_palette")
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -60.0))
        .show(ctx, |ui| {
            egui::Frame::panel()
                .fill(egui::Color32::from_black_alpha(200))
                .rounding(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for i in 0..16 {
                            let color = Palette::get_color(i);
                            let response = ui.color_edit_button_srgba(color);
                            if response.clicked() {
                                *selected_color = i;
                            }
                        }
                    });
                });
        });

    // 2. Кнопка Save (Bottom Right)
    egui::Area::new("save_button")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
        .show(ctx, |ui| {
            if ui.button("💾 Save Changes").clicked() {
                on_save();
            }
        });
}