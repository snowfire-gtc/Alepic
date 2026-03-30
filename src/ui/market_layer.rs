use eframe::egui;

pub fn render_market_layer(ctx: &egui::Context, on_wallet: &mut dyn FnMut()) {
    // Кнопка Wallet (Bottom Right)
    egui::Area::new("wallet_button")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
        .show(ctx, |ui| {
            if ui.button("👛 Connect Wallet").clicked() {
                on_wallet();
            }
        });

    // Отображение цен на чанки (при наведении)
    // Реализуется через Tooltip в основном цикле рендеринга
}