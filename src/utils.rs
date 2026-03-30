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

//! Utility functions for Alepic
//! 
//! Provides:
//! - 4-bit color palette conversion
//! - Coordinate system transformations
//! - Chunk ID calculations

use eframe::egui;

/// Total canvas dimensions (4K UHD)
pub const CANVAS_WIDTH: u32 = 4096;
pub const CANVAS_HEIGHT: u32 = 2160;

/// Chunk dimensions
pub const CHUNK_SIZE: u32 = 16;
pub const GRID_WIDTH: u32 = CANVAS_WIDTH / CHUNK_SIZE; // 256
pub const GRID_HEIGHT: u32 = CANVAS_HEIGHT / CHUNK_SIZE; // 135
pub const TOTAL_CHUNKS: u32 = GRID_WIDTH * GRID_HEIGHT; // 34,560

/// 4-bit color palette (16 colors)
/// Bright colors (0-7), Dark colors (8-15)
pub const PALETTE: [[u8; 3]; 16] = [
    // Bright colors
    [255, 255, 255], // 0: White
    [255, 255, 0],   // 1: Yellow
    [255, 128, 0],   // 2: Orange
    [255, 0, 128],   // 3: Pink
    [255, 0, 0],     // 4: Red
    [128, 0, 255],   // 5: Purple
    [0, 0, 255],     // 6: Blue
    [0, 255, 255],   // 7: Cyan
    // Dark colors
    [128, 128, 128], // 8: Gray
    [128, 128, 0],   // 9: Dark Yellow
    [128, 64, 0],    // 10: Dark Orange
    [128, 0, 64],    // 11: Dark Pink
    [128, 0, 0],     // 12: Dark Red
    [64, 0, 128],    // 13: Dark Purple
    [0, 0, 128],     // 14: Dark Blue
    [0, 128, 128],   // 15: Dark Cyan
];

/// Convert 4-bit color index to RGB
pub fn color_to_rgb(color_index: u8) -> [u8; 3] {
    PALETTE[color_index as usize % 16]
}

/// Convert RGB to egui Color32
pub fn color_to_egui(color_index: u8) -> egui::Color32 {
    let rgb = color_to_rgb(color_index);
    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

/// Convert pixel coordinates to chunk ID
pub fn pixel_to_chunk_id(x: u32, y: u32) -> Option<u32> {
    if x >= CANVAS_WIDTH || y >= CANVAS_HEIGHT {
        return None;
    }
    let chunk_x = x / CHUNK_SIZE;
    let chunk_y = y / CHUNK_SIZE;
    Some(chunk_y * GRID_WIDTH + chunk_x)
}

/// Convert chunk ID to grid coordinates
pub fn chunk_id_to_grid(chunk_id: u32) -> Option<(u32, u32)> {
    if chunk_id >= TOTAL_CHUNKS {
        return None;
    }
    let grid_x = chunk_id % GRID_WIDTH;
    let grid_y = chunk_id / GRID_WIDTH;
    Some((grid_x, grid_y))
}

/// Convert chunk ID to pixel coordinates (top-left corner)
pub fn chunk_id_to_pixels(chunk_id: u32) -> Option<(u32, u32)> {
    chunk_id_to_grid(chunk_id).map(|(gx, gy)| {
        (gx * CHUNK_SIZE, gy * CHUNK_SIZE)
    })
}

/// Get chunk ID from screen position
pub fn screen_to_chunk_id(
    screen_pos: egui::Pos2,
    viewport_offset: egui::Vec2,
    zoom: f32,
) -> Option<u32> {
    let world_x = (screen_pos.x - viewport_offset.x) / zoom;
    let world_y = (screen_pos.y - viewport_offset.y) / zoom;
    
    if world_x < 0.0 || world_y < 0.0 {
        return None;
    }
    
    let pixel_x = (world_x as u32).min(CANVAS_WIDTH - 1);
    let pixel_y = (world_y as u32).min(CANVAS_HEIGHT - 1);
    
    pixel_to_chunk_id(pixel_x, pixel_y)
}

/// Calculate distance between two chunk IDs
pub fn chunk_distance(chunk_a: u32, chunk_b: u32) -> Option<f32> {
    let (ax, ay) = chunk_id_to_grid(chunk_a)?;
    let (bx, by) = chunk_id_to_grid(chunk_b)?;
    
    let dx = (ax as i32 - bx as i32) as f32;
    let dy = (ay as i32 - by as i32) as f32;
    
    Some((dx * dx + dy * dy).sqrt())
}

/// Get neighboring chunk IDs (8 directions)
pub fn get_neighbors(chunk_id: u32) -> Vec<u32> {
    let mut neighbors = Vec::with_capacity(8);
    
    if let Some((gx, gy)) = chunk_id_to_grid(chunk_id) {
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                
                if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                    let neighbor_id = (ny as u32 * GRID_WIDTH) + (nx as u32);
                    neighbors.push(neighbor_id);
                }
            }
        }
    }
    
    neighbors
}

/// Check if a chunk is within Alepe's jump range
pub fn is_in_alepe_range(chunk_id: u32, alepe_x: u32, alepe_y: u32, min_dist: u32, max_dist: u32) -> bool {
    if let Some((px, py)) = chunk_id_to_pixels(chunk_id) {
        let chunk_center_x = px + CHUNK_SIZE / 2;
        let chunk_center_y = py + CHUNK_SIZE / 2;
        
        let dx = (chunk_center_x as i32 - alepe_x as i32).abs() as u32;
        let dy = (chunk_center_y as i32 - alepe_y as i32).abs() as u32;
        
        let distance = ((dx * dx + dy * dy) as f32).sqrt() as u32;
        
        distance >= min_dist && distance <= max_dist
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pixel_to_chunk_conversion() {
        assert_eq!(pixel_to_chunk_id(0, 0), Some(0));
        assert_eq!(pixel_to_chunk_id(15, 15), Some(0));
        assert_eq!(pixel_to_chunk_id(16, 0), Some(1));
        assert_eq!(pixel_to_chunk_id(0, 16), Some(GRID_WIDTH));
    }
    
    #[test]
    fn test_chunk_to_pixel_conversion() {
        assert_eq!(chunk_id_to_pixels(0), Some((0, 0)));
        assert_eq!(chunk_id_to_pixels(1), Some((16, 0)));
        assert_eq!(chunk_id_to_pixels(GRID_WIDTH), Some((0, 16)));
    }
    
    #[test]
    fn test_color_palette() {
        let white = color_to_rgb(0);
        assert_eq!(white, [255, 255, 255]);
        
        let blue = color_to_rgb(6);
        assert_eq!(blue, [0, 0, 255]);
    }
}
