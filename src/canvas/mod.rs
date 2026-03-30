pub mod chunk;

use crate::canvas::chunk::{Chunk, CHUNK_SIZE, ColorIndex};
use std::collections::HashMap;

/// Canvas manager for handling all chunks and pixel operations
pub struct CanvasManager {
    chunks: HashMap<u32, Chunk>,
    width_chunks: u16,
    height_chunks: u16,
}

impl CanvasManager {
    pub fn new() -> Self {
        // 4096 x 2160 pixels = 256 x 135 chunks (16x16 pixels each)
        Self {
            chunks: HashMap::new(),
            width_chunks: 256,
            height_chunks: 135,
        }
    }

    /// Get or create a chunk by grid coordinates
    pub fn get_or_create_chunk(&mut self, grid_x: u16, grid_y: u16) -> &mut Chunk {
        let chunk_id = Self::grid_to_id(grid_x, grid_y);
        
        if !self.chunks.contains_key(&chunk_id) {
            let chunk = Chunk::new(chunk_id, grid_x, grid_y);
            self.chunks.insert(chunk_id, chunk);
        }
        
        // Safe unwrap: we just inserted the chunk if it didn't exist
        self.chunks.get_mut(&chunk_id).expect("Chunk should exist after insertion")
    }

    /// Get chunk by ID
    pub fn get_chunk(&self, chunk_id: u32) -> Option<&Chunk> {
        self.chunks.get(&chunk_id)
    }

    /// Get mutable chunk by ID
    pub fn get_chunk_mut(&mut self, chunk_id: u32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&chunk_id)
    }

    /// Set a pixel on the canvas (by global pixel coordinates)
    pub fn set_pixel(&mut self, pixel_x: u32, pixel_y: u32, color: ColorIndex) -> Option<u32> {
        if pixel_x >= (self.width_chunks as u32 * CHUNK_SIZE as u32) ||
           pixel_y >= (self.height_chunks as u32 * CHUNK_SIZE as u32) {
            return None;
        }

        let grid_x = (pixel_x / CHUNK_SIZE as u32) as u16;
        let grid_y = (pixel_y / CHUNK_SIZE as u32) as u16;
        let local_x = (pixel_x % CHUNK_SIZE as u32) as u16;
        let local_y = (pixel_y % CHUNK_SIZE as u32) as u16;

        let chunk = self.get_or_create_chunk(grid_x, grid_y);
        chunk.set_pixel(local_x, local_y, color);
        
        Some(chunk.id)
    }

    /// Get a pixel color from the canvas
    pub fn get_pixel(&self, pixel_x: u32, pixel_y: u32) -> Option<ColorIndex> {
        if pixel_x >= (self.width_chunks as u32 * CHUNK_SIZE as u32) ||
           pixel_y >= (self.height_chunks as u32 * CHUNK_SIZE as u32) {
            return None;
        }

        let grid_x = (pixel_x / CHUNK_SIZE as u32) as u16;
        let grid_y = (pixel_y / CHUNK_SIZE as u32) as u16;
        let local_x = (pixel_x % CHUNK_SIZE as u32) as u16;
        let local_y = (pixel_y % CHUNK_SIZE as u32) as u16;

        let chunk_id = Self::grid_to_id(grid_x, grid_y);
        self.chunks.get(&chunk_id).map(|chunk| {
            let idx = (local_y * CHUNK_SIZE + local_x) as usize;
            chunk.pixels[idx]
        })
    }

    /// Convert grid coordinates to chunk ID
    pub fn grid_to_id(grid_x: u16, grid_y: u16) -> u32 {
        (grid_y as u32 * 256) + grid_x as u32
    }

    /// Convert chunk ID to grid coordinates
    pub fn id_to_grid(chunk_id: u32) -> (u16, u16) {
        let grid_x = (chunk_id % 256) as u16;
        let grid_y = (chunk_id / 256) as u16;
        (grid_x, grid_y)
    }

    /// Get all dirty chunks (with pending changes)
    pub fn get_dirty_chunks(&self) -> Vec<&Chunk> {
        self.chunks.values().filter(|c| c.is_dirty).collect()
    }

    /// Get total number of chunks
    pub fn total_chunks(&self) -> u32 {
        self.width_chunks as u32 * self.height_chunks as u32
    }

    /// Get canvas dimensions in chunks
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width_chunks, self.height_chunks)
    }

    /// Get canvas dimensions in pixels
    pub fn pixel_dimensions(&self) -> (u32, u32) {
        (
            self.width_chunks as u32 * CHUNK_SIZE as u32,
            self.height_chunks as u32 * CHUNK_SIZE as u32,
        )
    }
}

impl Default for CanvasManager {
    fn default() -> Self {
        Self::new()
    }
}
