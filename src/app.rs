use eframe::egui;
use crate::canvas::chunk::{Chunk, CHUNK_SIZE};
use crate::rendering::texture_mgr::TextureManager;
use crate::ui::layers::{LayerType, LayerUI};
use crate::game::alepe::Alepe;
use crate::blockchain::wallet::WalletManager;

pub struct AlepicApp {
    chunks: HashMap<u32, Chunk>,
    texture_mgr: TextureManager,
    viewport: Viewport,
    current_layer: LayerType,
    alepe: Alepe,
    wallet: WalletManager,
    selected_color: u8,
    pending_changes: Vec<PixelChange>,
}

struct Viewport {
    zoom: f32,
    offset: egui::Vec2,
}

struct PixelChange {
    chunk_id: u32,
    x: u16,
    y: u16,
    color: u8,
}

impl eframe::App for AlepicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Обработка ввода (Zoom/Pan)
        self.handle_input(ctx);

        // 2. Отрисовка Холста (Центральная панель)
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_canvas(ui);
        });

        // 3. Отрисовка UI Слоев
        self.render_layer_ui(ctx);

        // 4. Проверка событий Alepe
        self.alepe.check_jump(ctx);
    }
}

impl AlepicApp {
    fn handle_input(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Zoom: Mouse Wheel
        if let Some(delta) = input.scroll_delta.y {
            self.viewport.zoom *= 1.0 + (delta * 0.001);
            self.viewport.zoom = self.viewport.zoom.clamp(0.5, 10.0);
        }

        // Pan: Left Mouse Drag
        if input.pointer.button_down(egui::PointerButton::Primary) {
            if let Some(delta) = input.pointer.delta() {
                // Только если не кликаем по UI элементам
                self.viewport.offset += delta;
            }
        }
    }

    fn render_canvas(&mut self, ui: &egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();

        // Расчет видимых чанков на основе вьюпорта
        let visible_chunks = self.calculate_visible_chunks(available_rect);

        for chunk_id in &visible_chunks {
            if let Some(chunk) = self.chunks.get(chunk_id) {
                // Обновление текстуры при необходимости
                self.texture_mgr.update_chunk(chunk);

                // Отрисовка изображения чанка
                if let Some(texture) = self.texture_mgr.get_texture(*chunk_id) {
                    let pos = self.get_chunk_screen_pos(chunk, available_rect);
                    let size = egui::vec2(16.0, 16.0) * self.viewport.zoom;

                    let mut image = egui::Image::new(texture, size);

                    // Подсветка при наведении
                    if ui.is_rect_hovered(pos) {
                        image = image.tint(egui::Color32::from_white_alpha(100));
                    }

                    ui.put(pos, image);
                }
            }
        }

        // Очистка невидимых текстур
        self.texture_mgr.cleanup(&visible_chunks);

        // Отрисовка Alepe (если виден)
        self.alepe.render(ui, &self.viewport, available_rect);
    }

    fn render_layer_ui(&mut self, ctx: &egui::Context) {
        match self.current_layer {
            LayerType::Color => self.render_color_layer(ctx),
            LayerType::Market => self.render_market_layer(ctx),
        }
    }
}
