use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: u16 = 16;
pub const PIXELS_PER_CHUNK: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

/// 4-bit color index (0-15)
pub type ColorIndex = u8;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: u32,
    pub grid_x: u16,
    pub grid_y: u16,
    pub owner: Option<String>, // Alephium Address
    pub pixels: [ColorIndex; PIXELS_PER_CHUNK],
    pub version: u64, // Для проверки актуальности
    pub is_dirty: bool, // Локальные изменения
}

impl Chunk {
    pub fn new(id: u32, grid_x: u16, grid_y: u16) -> Self {
        Self {
            id,
            grid_x,
            grid_y,
            owner: None,
            pixels: [0; PIXELS_PER_CHUNK], // Default color (index 0)
            version: 0,
            is_dirty: false,
        }
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: ColorIndex) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            let idx = (y * CHUNK_SIZE + x) as usize;
            if self.pixels[idx] != color {
                self.pixels[idx] = color;
                self.is_dirty = true;
            }
        }
    }

    pub fn to_texture_data(&self) -> Vec<u8> {
        // Конвертация индексов в RGBA для GPU (упрощенно)
        // В реальности используется lookup-таблица палитры
        self.pixels.iter().flat_map(|&idx| {
            let color = Palette::get_color(idx);
            vec![color.r, color.g, color.b, 255]
        }).collect()
    }
}

pub struct Palette;

impl Palette {
    /// 4-bit Indexed Palette (16 colors) - Section 3.1
    pub fn get_color(index: u8) -> egui::Color32 {
        const PALETTE: [(u8, u8, u8); 16] = [
            (0, 0, 0), (255, 255, 255), (255, 0, 0), (0, 255, 0),
            (0, 0, 255), (255, 255, 0), (0, 255, 255), (255, 0, 255),
            (128, 128, 128), (192, 192, 192), (128, 0, 0), (0, 128, 0),
            (0, 0, 128), (128, 128, 0), (0, 128, 128), (128, 0, 128),
        ];
        let (r, g, b) = PALETTE[index as usize];
        egui::Color32::from_rgb(r, g, b)
    }
}