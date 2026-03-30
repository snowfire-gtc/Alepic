use eframe::egui;
use rand::Rng;

// Constants from README.md
const MIN_DISTANCE: u16 = 48; // pixels
const MAX_DISTANCE: u16 = 96; // pixels
const JUMP_INTERVAL_BLOCKS: u64 = 100_000;
const CHUNK_SIZE_PIXELS: u16 = 16;
const CANVAS_WIDTH_CHUNKS: u16 = 256;
const CANVAS_HEIGHT_CHUNKS: u16 = 135;

pub struct Alepe {
    pub grid_x: u16, // In chunks (0-255)
    pub grid_y: u16, // In chunks (0-134)
    pub last_jump_block: u64,
    pub pixel_offset_x: u16, // Offset within chunk (0-15)
    pub pixel_offset_y: u16, // Offset within chunk (0-15)
}

impl Alepe {
    pub fn new() -> Self {
        Self {
            grid_x: CANVAS_WIDTH_CHUNKS / 2,   // Center: 128
            grid_y: CANVAS_HEIGHT_CHUNKS / 2,  // Center: 67
            last_jump_block: 0,
            pixel_offset_x: 0,
            pixel_offset_y: 0,
        }
    }

    /// Check if Alepe should jump based on current block number
    /// Returns true if a jump occurred
    pub fn check_jump(&mut self, current_block: u64) -> bool {
        if current_block >= self.last_jump_block + JUMP_INTERVAL_BLOCKS {
            self.jump(current_block);
            true
        } else {
            false
        }
    }

    /// Perform Alepe's jump according to the pseudocode from README.md
    fn jump(&mut self, current_block: u64) {
        let mut rng = rand::thread_rng();
        
        // Calculate distance in pixels (48-96 pixels = 3-6 chunks)
        let dx_pixels = rng.gen_range(MIN_DISTANCE..=MAX_DISTANCE);
        let dy_pixels = rng.gen_range(MIN_DISTANCE..=MAX_DISTANCE);
        
        // Random direction signs
        let sign_x: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
        let sign_y: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
        
        // Convert pixel distance to chunk distance
        let dx_chunks = dx_pixels / CHUNK_SIZE_PIXELS;
        let dy_chunks = dy_pixels / CHUNK_SIZE_PIXELS;
        
        // Apply movement with wrapping
        if sign_x > 0 {
            self.grid_x = (self.grid_x as u32 + dx_chunks as u32) % CANVAS_WIDTH_CHUNKS as u32 as u16;
        } else {
            self.grid_x = (self.grid_x as i32 - dx_chunks as i32).rem_euclid(CANVAS_WIDTH_CHUNKS as i32) as u16;
        }
        
        if sign_y > 0 {
            self.grid_y = (self.grid_y as u32 + dy_chunks as u32) % CANVAS_HEIGHT_CHUNKS as u32 as u16;
        } else {
            self.grid_y = (self.grid_y as i32 - dy_chunks as i32).rem_euclid(CANVAS_HEIGHT_CHUNKS as i32) as u16;
        }
        
        // Update pixel offset within the chunk (for precise position)
        self.pixel_offset_x = dx_pixels % CHUNK_SIZE_PIXELS;
        self.pixel_offset_y = dy_pixels % CHUNK_SIZE_PIXELS;
        
        self.last_jump_block = current_block;
        
        println!("Alepe jumped to chunk ({}, {}) at block {}", 
                 self.grid_x, self.grid_y, current_block);
    }

    /// Get the exact pixel position of Alepe
    pub fn get_pixel_position(&self) -> (u32, u32) {
        let pixel_x = (self.grid_x as u32 * CHUNK_SIZE_PIXELS as u32) + self.pixel_offset_x as u32;
        let pixel_y = (self.grid_y as u32 * CHUNK_SIZE_PIXELS as u32) + self.pixel_offset_y as u32;
        (pixel_x, pixel_y)
    }

    /// Check if a chunk is occupied by Alepe (2x2 chunks)
    pub fn occupies_chunk(&self, chunk_grid_x: u16, chunk_grid_y: u16) -> bool {
        // Alepe occupies 2x2 chunks centered on her position
        let min_x = self.grid_x.saturating_sub(1);
        let max_x = (self.grid_x + 1).min(CANVAS_WIDTH_CHUNKS - 1);
        let min_y = self.grid_y.saturating_sub(1);
        let max_y = (self.grid_y + 1).min(CANVAS_HEIGHT_CHUNKS - 1);
        
        chunk_grid_x >= min_x && chunk_grid_x <= max_x &&
        chunk_grid_y >= min_y && chunk_grid_y <= max_y
    }

    /// Get chunks around Alepe that should be auction chunks
    pub fn get_auction_chunks(&self) -> Vec<(u16, u16)> {
        let mut auction_chunks = Vec::new();
        
        // Auction chunks are unowned chunks around Alepe's position
        for dx in -2..=2i16 {
            for dy in -2..=2i16 {
                if dx == 0 && dy == 0 {
                    continue; // Skip Alepe's own chunks
                }
                
                let cx = (self.grid_x as i16 + dx).rem_euclid(CANVAS_WIDTH_CHUNKS as i16) as u16;
                let cy = (self.grid_y as i16 + dy).rem_euclid(CANVAS_HEIGHT_CHUNKS as i16) as u16;
                
                auction_chunks.push((cx, cy));
            }
        }
        
        auction_chunks
    }

    /// Render Alepe on the canvas
    pub fn render(&self, ui: &egui::Ui, viewport: &crate::app::Viewport, rect: egui::Rect) {
        // Alepe occupies 2x2 chunks (32x32 pixels in base size)
        let base_x = rect.min.x + viewport.offset.x;
        let base_y = rect.min.y + viewport.offset.y;
        let alepe_x = base_x + (self.grid_x as f32 * 16.0 * viewport.zoom);
        let alepe_y = base_y + (self.grid_y as f32 * 16.0 * viewport.zoom);
        let alepe_size = egui::vec2(32.0 * viewport.zoom, 32.0 * viewport.zoom);

        let alepe_rect = egui::Rect::from_min_size(
            egui::pos2(alepe_x, alepe_y),
            alepe_size,
        );

        // Draw frame around Alepe's position
        ui.painter().rect_stroke(
            alepe_rect,
            5.0,
            egui::Stroke::new(2.0, egui::Color32::GREEN),
        );

        // Draw Alepe label
        ui.painter().text(
            alepe_rect.center_top(),
            egui::Align2::CENTER_BOTTOM,
            "🐸 Alepe",
            egui::FontId::default(),
            egui::Color32::GREEN,
        );
    }

    /// Get blocks until next jump
    pub fn blocks_until_jump(&self, current_block: u64) -> u64 {
        let next_jump_block = self.last_jump_block + JUMP_INTERVAL_BLOCKS;
        if current_block >= next_jump_block {
            0
        } else {
            next_jump_block - current_block
        }
    }
}

impl Default for Alepe {
    fn default() -> Self {
        Self::new()
    }
}