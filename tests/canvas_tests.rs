// SPDX-License-Identifier: GPL-3.0
// Copyright (C) 2026 Sergey Antonov
//
// This file is part of Alepic (Alephium Collaborative Canvas).
//
// Alepic is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Alepic is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Comprehensive test suite for Alepic Canvas System
//! 
//! Tests cover:
//! - Chunk creation and management
//! - Pixel operations
//! - Coordinate transformations
//! - Canvas boundaries
//! - Dirty flag tracking

#[cfg(test)]
mod tests {
    use crate::canvas::chunk::{Chunk, CHUNK_SIZE, PIXELS_PER_CHUNK, ColorIndex};
    use crate::canvas::CanvasManager;

    // ==================== Chunk Tests ====================

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new(42, 5, 10);
        
        assert_eq!(chunk.id, 42);
        assert_eq!(chunk.grid_x, 5);
        assert_eq!(chunk.grid_y, 10);
        assert_eq!(chunk.owner, None);
        assert_eq!(chunk.version, 0);
        assert!(!chunk.is_dirty);
        assert_eq!(chunk.pixels.len(), PIXELS_PER_CHUNK);
        
        // All pixels should be initialized to 0 (default color)
        for &pixel in &chunk.pixels {
            assert_eq!(pixel, 0);
        }
    }

    #[test]
    fn test_chunk_set_pixel() {
        let mut chunk = Chunk::new(1, 0, 0);
        
        // Set a pixel
        chunk.set_pixel(5, 7, 8);
        
        // Verify pixel was set
        let idx = (7 * CHUNK_SIZE + 5) as usize;
        assert_eq!(chunk.pixels[idx], 8);
        assert!(chunk.is_dirty);
    }

    #[test]
    fn test_chunk_set_pixel_out_of_bounds() {
        let mut chunk = Chunk::new(1, 0, 0);
        let original_pixels = chunk.pixels.clone();
        
        // Try to set pixels outside bounds
        chunk.set_pixel(CHUNK_SIZE, 0, 5);  // x out of bounds
        chunk.set_pixel(0, CHUNK_SIZE, 5);  // y out of bounds
        chunk.set_pixel(20, 20, 5);         // both out of bounds
        
        // Pixels should remain unchanged
        assert_eq!(chunk.pixels, original_pixels);
        assert!(!chunk.is_dirty);
    }

    #[test]
    fn test_chunk_set_same_color_no_dirty() {
        let mut chunk = Chunk::new(1, 0, 0);
        
        // Set a pixel to color 5
        chunk.set_pixel(3, 3, 5);
        assert!(chunk.is_dirty);
        
        // Reset dirty flag
        chunk.is_dirty = false;
        
        // Set same pixel to same color - should not mark as dirty
        chunk.set_pixel(3, 3, 5);
        assert!(!chunk.is_dirty);
    }

    #[test]
    fn test_chunk_all_pixels() {
        let mut chunk = Chunk::new(1, 0, 0);
        
        // Set all pixels to different colors
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let color = ((x + y) % 16) as ColorIndex;
                chunk.set_pixel(x, y, color);
            }
        }
        
        // Verify all pixels
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let idx = (y * CHUNK_SIZE + x) as usize;
                let expected = ((x + y) % 16) as ColorIndex;
                assert_eq!(chunk.pixels[idx], expected);
            }
        }
    }

    #[test]
    fn test_chunk_texture_data() {
        let mut chunk = Chunk::new(1, 0, 0);
        chunk.set_pixel(0, 0, 1);
        chunk.set_pixel(1, 0, 2);
        
        let texture_data = chunk.to_texture_data();
        
        // Each pixel becomes 4 bytes (RGBA)
        assert_eq!(texture_data.len(), PIXELS_PER_CHUNK * 4);
        
        // First pixel should be yellow (255, 255, 0)
        assert_eq!(texture_data[0], 255); // R
        assert_eq!(texture_data[1], 255); // G
        assert_eq!(texture_data[2], 0);   // B
        assert_eq!(texture_data[3], 255); // A
    }

    // ==================== Canvas Manager Tests ====================

    #[test]
    fn test_canvas_manager_creation() {
        let canvas = CanvasManager::new();
        
        let (width_chunks, height_chunks) = canvas.dimensions();
        assert_eq!(width_chunks, 256);
        assert_eq!(height_chunks, 135);
        
        assert_eq!(canvas.total_chunks(), 34560);
        
        let (width_px, height_px) = canvas.pixel_dimensions();
        assert_eq!(width_px, 4096);
        assert_eq!(height_px, 2160);
    }

    #[test]
    fn test_canvas_get_or_create_chunk() {
        let mut canvas = CanvasManager::new();
        
        // Get chunk that doesn't exist yet
        let chunk = canvas.get_or_create_chunk(10, 20);
        
        assert_eq!(chunk.grid_x, 10);
        assert_eq!(chunk.grid_y, 20);
        
        // Same chunk should be returned on second call
        let chunk2 = canvas.get_or_create_chunk(10, 20);
        assert_eq!(chunk2.id, chunk.id);
    }

    #[test]
    fn test_canvas_grid_to_id_conversion() {
        // Test known conversions
        assert_eq!(CanvasManager::grid_to_id(0, 0), 0);
        assert_eq!(CanvasManager::grid_to_id(1, 0), 1);
        assert_eq!(CanvasManager::grid_to_id(0, 1), 256);
        assert_eq!(CanvasManager::grid_to_id(255, 134), 34559);
    }

    #[test]
    fn test_canvas_id_to_grid_conversion() {
        // Test round-trip conversion
        for grid_x in [0, 1, 128, 255] {
            for grid_y in [0, 1, 67, 134] {
                let id = CanvasManager::grid_to_id(grid_x, grid_y);
                let (back_x, back_y) = CanvasManager::id_to_grid(id);
                assert_eq!(back_x, grid_x);
                assert_eq!(back_y, grid_y);
            }
        }
    }

    #[test]
    fn test_canvas_set_pixel() {
        let mut canvas = CanvasManager::new();
        
        // Set pixel at global coordinates
        let result = canvas.set_pixel(100, 200, 7);
        
        assert!(result.is_some());
        let chunk_id = result.unwrap();
        
        // Verify pixel was set correctly
        let pixel = canvas.get_pixel(100, 200);
        assert_eq!(pixel, Some(7));
    }

    #[test]
    fn test_canvas_set_pixel_out_of_bounds() {
        let mut canvas = CanvasManager::new();
        
        // Try to set pixels outside canvas
        assert_eq!(canvas.set_pixel(4096, 100, 5), None);  // x out of bounds
        assert_eq!(canvas.set_pixel(100, 2160, 5), None);  // y out of bounds
        assert_eq!(canvas.set_pixel(5000, 3000, 5), None); // both out
    }

    #[test]
    fn test_canvas_get_pixel_nonexistent_chunk() {
        let canvas = CanvasManager::new();
        
        // Get pixel from chunk that doesn't exist yet
        let pixel = canvas.get_pixel(100, 100);
        
        // Should return None since chunk doesn't exist
        assert_eq!(pixel, None);
    }

    #[test]
    fn test_canvas_dirty_chunks() {
        let mut canvas = CanvasManager::new();
        
        // Initially no dirty chunks
        assert_eq!(canvas.get_dirty_chunks().len(), 0);
        
        // Set some pixels
        canvas.set_pixel(10, 10, 1);
        canvas.set_pixel(50, 50, 2);
        canvas.set_pixel(100, 100, 3);
        
        // Should have dirty chunks now
        let dirty = canvas.get_dirty_chunks();
        assert_eq!(dirty.len(), 3);
    }

    #[test]
    fn test_canvas_chunk_boundaries() {
        let mut canvas = CanvasManager::new();
        
        // Test pixels at chunk boundaries
        // Chunk (0,0): pixels 0-15, 0-15
        canvas.set_pixel(15, 15, 1);
        canvas.set_pixel(16, 15, 2);  // First pixel of chunk (1,0)
        canvas.set_pixel(15, 16, 3);  // First pixel of chunk (0,1)
        canvas.set_pixel(16, 16, 4);  // First pixel of chunk (1,1)
        
        assert_eq!(canvas.get_pixel(15, 15), Some(1));
        assert_eq!(canvas.get_pixel(16, 15), Some(2));
        assert_eq!(canvas.get_pixel(15, 16), Some(3));
        assert_eq!(canvas.get_pixel(16, 16), Some(4));
    }

    #[test]
    fn test_canvas_corner_pixels() {
        let mut canvas = CanvasManager::new();
        
        // Set pixels at all four corners
        canvas.set_pixel(0, 0, 1);                    // Top-left
        canvas.set_pixel(4095, 0, 2);                 // Top-right
        canvas.set_pixel(0, 2159, 3);                 // Bottom-left
        canvas.set_pixel(4095, 2159, 4);              // Bottom-right
        
        assert_eq!(canvas.get_pixel(0, 0), Some(1));
        assert_eq!(canvas.get_pixel(4095, 0), Some(2));
        assert_eq!(canvas.get_pixel(0, 2159), Some(3));
        assert_eq!(canvas.get_pixel(4095, 2159), Some(4));
    }

    #[test]
    fn test_canvas_multiple_pixels_same_chunk() {
        let mut canvas = CanvasManager::new();
        
        // Set multiple pixels in the same chunk
        for i in 0..10 {
            canvas.set_pixel(i, i, i as u8);
        }
        
        // Verify all pixels
        for i in 0..10 {
            assert_eq!(canvas.get_pixel(i, i), Some(i as u8));
        }
        
        // Should only have 1 dirty chunk
        assert_eq!(canvas.get_dirty_chunks().len(), 1);
    }

    #[test]
    fn test_canvas_get_chunk_by_id() {
        let mut canvas = CanvasManager::new();
        
        // Create a chunk by setting a pixel
        canvas.set_pixel(100, 100, 5);
        
        // Get chunk ID
        let chunk_id = CanvasManager::grid_to_id(6, 6); // 100/16 = 6
        
        // Retrieve chunk
        let chunk = canvas.get_chunk(chunk_id);
        assert!(chunk.is_some());
        assert_eq!(chunk.unwrap().grid_x, 6);
        assert_eq!(chunk.unwrap().grid_y, 6);
    }

    #[test]
    fn test_canvas_get_nonexistent_chunk() {
        let canvas = CanvasManager::new();
        
        // Try to get chunk that doesn't exist
        let chunk = canvas.get_chunk(999);
        assert_eq!(chunk, None);
    }

    #[test]
    fn test_canvas_modify_chunk_via_mut() {
        let mut canvas = CanvasManager::new();
        
        // Create chunk
        canvas.set_pixel(50, 50, 1);
        
        // Get mutable chunk and modify directly
        let chunk_id = CanvasManager::grid_to_id(3, 3);
        if let Some(chunk) = canvas.get_chunk_mut(chunk_id) {
            chunk.set_pixel(0, 0, 15);
            chunk.version = 42;
        }
        
        // Verify changes
        let chunk = canvas.get_chunk(chunk_id).unwrap();
        assert_eq!(chunk.pixels[0], 15);
        assert_eq!(chunk.version, 42);
    }
}
