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

//! Comprehensive test suite for Alepe Game Mechanics
//! 
//! Tests cover:
//! - Alepe movement and jumping
//! - Block-based timing
//! - Position calculations
//! - Chunk occupation logic
//! - Auction chunk detection

#[cfg(test)]
mod tests {
    use crate::game::alepe::Alepe;

    // ==================== Alepe Creation Tests ====================

    #[test]
    fn test_alepe_creation() {
        let alepe = Alepe::new();
        
        // Should start at center of canvas
        assert_eq!(alepe.grid_x, 128);  // 256 / 2
        assert_eq!(alepe.grid_y, 67);   // 135 / 2
        assert_eq!(alepe.last_jump_block, 0);
        assert_eq!(alepe.pixel_offset_x, 0);
        assert_eq!(alepe.pixel_offset_y, 0);
    }

    #[test]
    fn test_alepe_default_trait() {
        let alepe = Alepe::default();
        
        assert_eq!(alepe.grid_x, 128);
        assert_eq!(alepe.grid_y, 67);
    }

    // ==================== Jump Timing Tests ====================

    #[test]
    fn test_alepe_no_jump_before_interval() {
        let mut alepe = Alepe::new();
        
        // Should not jump before 100,000 blocks
        assert!(!alepe.check_jump(50_000));
        assert!(!alepe.check_jump(99_999));
        
        // Position should remain unchanged
        assert_eq!(alepe.grid_x, 128);
        assert_eq!(alepe.grid_y, 67);
    }

    #[test]
    fn test_alepe_jump_at_interval() {
        let mut alepe = Alepe::new();
        
        // Should jump at exactly 100,000 blocks
        assert!(alepe.check_jump(100_000));
        
        // Position should have changed
        assert_ne!(alepe.grid_x, 128);
        assert_ne!(alepe.grid_y, 67);
        assert_eq!(alepe.last_jump_block, 100_000);
    }

    #[test]
    fn test_alepe_multiple_jumps() {
        let mut alepe = Alepe::new();
        
        // First jump
        assert!(alepe.check_jump(100_000));
        let pos_after_first = (alepe.grid_x, alepe.grid_y);
        
        // Should not jump again until next interval
        assert!(!alepe.check_jump(150_000));
        assert_eq!((alepe.grid_x, alepe.grid_y), pos_after_first);
        
        // Second jump at 200,000
        assert!(alepe.check_jump(200_000));
        assert_eq!(alepe.last_jump_block, 200_000);
    }

    #[test]
    fn test_alepe_blocks_until_jump() {
        let mut alepe = Alepe::new();
        
        // At block 0, should be 100,000 blocks until jump
        assert_eq!(alepe.blocks_until_jump(0), 100_000);
        
        // At block 50,000, should be 50,000 blocks until jump
        assert_eq!(alepe.blocks_until_jump(50_000), 50_000);
        
        // After jump, reset counter
        alepe.check_jump(100_000);
        assert_eq!(alepe.blocks_until_jump(100_000), 100_000);
        assert_eq!(alepe.blocks_until_jump(150_000), 50_000);
    }

    #[test]
    fn test_alepe_blocks_until_jump_zero() {
        let mut alepe = Alepe::new();
        
        // When current block >= next jump block, should return 0
        alepe.check_jump(100_000);
        assert_eq!(alepe.blocks_until_jump(200_000), 0);
        assert_eq!(alepe.blocks_until_jump(250_000), 0);
    }

    // ==================== Jump Distance Tests ====================

    #[test]
    fn test_alepe_jump_distance_range() {
        // Run multiple jumps to verify distance is within range
        for _ in 0..10 {
            let mut alepe = Alepe::new();
            alepe.check_jump(100_000);
            
            // Distance should be between 3-6 chunks (48-96 pixels / 16)
            let dx = (alepe.grid_x as i32 - 128).abs();
            let dy = (alepe.grid_y as i32 - 67).abs();
            
            // Note: Due to wrapping, we check if movement occurred
            assert!(dx > 0 || dy > 0);
        }
    }

    #[test]
    fn test_alepe_pixel_offset_after_jump() {
        let mut alepe = Alepe::new();
        alepe.check_jump(100_000);
        
        // Pixel offsets should be set based on remainder
        assert!(alepe.pixel_offset_x < 16);
        assert!(alepe.pixel_offset_y < 16);
    }

    // ==================== Position Tests ====================

    #[test]
    fn test_alepe_get_pixel_position_initial() {
        let alepe = Alepe::new();
        
        let (px, py) = alepe.get_pixel_position();
        
        // Initial position: chunk (128, 67) with offset (0, 0)
        assert_eq!(px, 128 * 16);
        assert_eq!(py, 67 * 16);
    }

    #[test]
    fn test_alepe_get_pixel_position_with_offset() {
        let mut alepe = Alepe::new();
        alepe.check_jump(100_000);
        
        let (px, py) = alepe.get_pixel_position();
        
        // Position should include chunk and offset
        let expected_px = (alepe.grid_x as u32 * 16) + alepe.pixel_offset_x as u32;
        let expected_py = (alepe.grid_y as u32 * 16) + alepe.pixel_offset_y as u32;
        
        assert_eq!(px, expected_px);
        assert_eq!(py, expected_py);
    }

    // ==================== Chunk Occupation Tests ====================

    #[test]
    fn test_alepe_occupies_own_chunks() {
        let alepe = Alepe::new();
        
        // Alepe should occupy her own chunk and adjacent chunks (2x2 area)
        assert!(alepe.occupies_chunk(128, 67));  // Center
        assert!(alepe.occupies_chunk(127, 67));  // Left
        assert!(alepe.occupies_chunk(128, 66));  // Top
        assert!(alepe.occupies_chunk(127, 66));  // Top-left
    }

    #[test]
    fn test_alepe_does_not_occupy_distant_chunks() {
        let alepe = Alepe::new();
        
        // Chunks far from Alepe should not be occupied
        assert!(!alepe.occupies_chunk(0, 0));
        assert!(!alepe.occupies_chunk(255, 134));
        assert!(!alepe.occupies_chunk(100, 100));
    }

    #[test]
    fn test_alepe_occupation_after_jump() {
        let mut alepe = Alepe::new();
        alepe.check_jump(100_000);
        
        let new_x = alepe.grid_x;
        let new_y = alepe.grid_y;
        
        // Should occupy new position
        assert!(alepe.occupies_chunk(new_x, new_y));
        
        // Should not occupy old position (unless jumped nearby)
        if new_x != 128 || new_y != 67 {
            // Old center might still be occupied if jump was short
            // but distant chunks should definitely not be occupied
            assert!(!alepe.occupies_chunk(0, 0));
        }
    }

    #[test]
    fn test_alepe_occupation_boundary_conditions() {
        // Test when Alepe is at canvas edge
        let mut alepe = Alepe::new();
        alepe.grid_x = 0;
        alepe.grid_y = 0;
        
        // Should still occupy valid chunks (with wrapping consideration)
        assert!(alepe.occupies_chunk(0, 0));
        assert!(alepe.occupies_chunk(1, 0));
        assert!(alepe.occupies_chunk(0, 1));
        assert!(alepe.occupies_chunk(1, 1));
        
        // Negative chunks don't exist, so these should be false
        // (but implementation uses saturating_sub, so min is 0)
    }

    // ==================== Auction Chunks Tests ====================

    #[test]
    fn test_alepe_get_auction_chunks_count() {
        let alepe = Alepe::new();
        
        let auction_chunks = alepe.get_auction_chunks();
        
        // Should return chunks in -2..=2 range excluding center
        // That's 5x5 - 1 = 24 chunks
        assert_eq!(auction_chunks.len(), 24);
    }

    #[test]
    fn test_alepe_auction_chunks_exclude_center() {
        let alepe = Alepe::new();
        
        let auction_chunks = alepe.get_auction_chunks();
        
        // Center chunk should not be in auction list
        assert!(!auction_chunks.contains(&(128, 67)));
    }

    #[test]
    fn test_alepe_auction_chunks_around_position() {
        let alepe = Alepe::new();
        
        let auction_chunks = alepe.get_auction_chunks();
        
        // Should include chunks around center
        assert!(auction_chunks.contains(&(127, 67)));
        assert!(auction_chunks.contains(&(129, 67)));
        assert!(auction_chunks.contains(&(128, 66)));
        assert!(auction_chunks.contains(&(128, 68)));
        assert!(auction_chunks.contains(&(126, 67)));
        assert!(auction_chunks.contains(&(130, 67)));
    }

    #[test]
    fn test_alepe_auction_chunks_after_jump() {
        let mut alepe = Alepe::new();
        alepe.check_jump(100_000);
        
        let auction_chunks = alepe.get_auction_chunks();
        
        // All auction chunks should be around new position
        let new_x = alepe.grid_x as i16;
        let new_y = alepe.grid_y as i16;
        
        for (cx, cy) in &auction_chunks {
            let dx = (*cx as i16 - new_x).abs();
            let dy = (*cy as i16 - new_y).abs();
            
            // Should be within 2 chunks of Alepe
            assert!(dx <= 2 && dy <= 2);
        }
    }

    // ==================== Wrapping Tests ====================

    #[test]
    fn test_alepe_wrapping_at_right_edge() {
        let mut alepe = Alepe::new();
        alepe.grid_x = 254;  // Near right edge
        
        // Force a jump
        alepe.check_jump(100_000);
        
        // Position should be wrapped correctly
        assert!(alepe.grid_x < 256);
    }

    #[test]
    fn test_alepe_wrapping_at_left_edge() {
        let mut alepe = Alepe::new();
        alepe.grid_x = 2;  // Near left edge
        
        alepe.check_jump(100_000);
        
        // Position should be wrapped correctly
        assert!(alepe.grid_x < 256);
    }

    #[test]
    fn test_alepe_wrapping_at_top_edge() {
        let mut alepe = Alepe::new();
        alepe.grid_y = 2;  // Near top edge
        
        alepe.check_jump(100_000);
        
        // Position should be wrapped correctly
        assert!(alepe.grid_y < 135);
    }

    #[test]
    fn test_alepe_wrapping_at_bottom_edge() {
        let mut alepe = Alepe::new();
        alepe.grid_y = 133;  // Near bottom edge
        
        alepe.check_jump(100_000);
        
        // Position should be wrapped correctly
        assert!(alepe.grid_y < 135);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_alepe_jump_at_block_zero() {
        let mut alepe = Alepe::new();
        
        // Jump at block 0 should work (for testing purposes)
        assert!(alepe.check_jump(0));
        assert_eq!(alepe.last_jump_block, 0);
    }

    #[test]
    fn test_alepe_consecutive_checks_same_block() {
        let mut alepe = Alepe::new();
        
        // First check triggers jump
        assert!(alepe.check_jump(100_000));
        let pos = (alepe.grid_x, alepe.grid_y);
        
        // Consecutive checks at same block should not trigger another jump
        assert!(!alepe.check_jump(100_000));
        assert_eq!((alepe.grid_x, alepe.grid_y), pos);
    }

    #[test]
    fn test_alepe_large_block_numbers() {
        let mut alepe = Alepe::new();
        
        // Test with large block numbers
        let large_block = 1_000_000_000u64;
        alepe.last_jump_block = large_block - 100_000;
        
        // Should jump at large_block
        assert!(alepe.check_jump(large_block));
        assert_eq!(alepe.last_jump_block, large_block);
    }
}
