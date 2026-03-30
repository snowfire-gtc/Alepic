use crate::canvas::chunk::Chunk;
use std::collections::HashMap;

pub struct TextureManager {
    // Кэш текстур для видимых чанков
    chunk_textures: HashMap<u32, egui::TextureHandle>,
    ctx: egui::Context,
}

impl TextureManager {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            chunk_textures: HashMap::new(),
            ctx: ctx.clone(),
        }
    }

    pub fn update_chunk(&mut self, chunk: &Chunk) {
        // Обновляем текстуру только если чанк "грязный" или отсутствует
        if chunk.is_dirty || !self.chunk_textures.contains_key(&chunk.id) {
            let data = chunk.to_texture_data();
            let texture = self.ctx.load_texture(
                &format!("chunk_{}", chunk.id),
                egui::ColorImage::from_rgba_unmultiplied(
                    [16, 16], 
                    &data
                ),
                egui::TextureOptions::NEAREST, // Пиксель-арт стиль
            );
            self.chunk_textures.insert(chunk.id, texture);
        }
    }

    pub fn get_texture(&self, chunk_id: u32) -> Option<&egui::TextureHandle> {
        self.chunk_textures.get(&chunk_id)
    }

    pub fn cleanup(&mut self, visible_ids: &[u32]) {
        // Удаление текстур невидимых чанков для экономии памяти (Dynamic Loading)
        let to_remove: Vec<u32> = self.chunk_textures
            .keys()
            .filter(|id| !visible_ids.contains(id))
            .copied()
            .collect();
        
        for id in to_remove {
            self.chunk_textures.remove(&id);
        }
    }
}