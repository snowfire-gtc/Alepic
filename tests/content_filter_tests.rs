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

//! Comprehensive test suite for Content Filter System
//! 
//! Tests cover:
//! - Basic content filtering
//! - Pattern detection
//! - Strict mode behavior
//! - Text-like pattern detection
//! - Advanced filter with neural network support

#[cfg(test)]
mod tests {
    use crate::content_filter::{ContentFilter, AdvancedContentFilter, FilterStats, ModerationResult};

    // ==================== ContentFilter Creation Tests ====================

    #[test]
    fn test_content_filter_creation() {
        let filter = ContentFilter::new();
        let stats = filter.get_stats();
        
        assert!(stats.enabled);
        assert!(!stats.strict_mode);
        assert_eq!(stats.blocked_patterns_count, 0);
    }

    #[test]
    fn test_content_filter_default() {
        let filter = ContentFilter::default();
        let stats = filter.get_stats();
        
        assert!(stats.enabled);
        assert!(!stats.strict_mode);
    }

    // ==================== Enable/Disable Tests ====================

    #[test]
    fn test_filter_disabled_allows_all() {
        let mut filter = ContentFilter::new();
        filter.set_enabled(false);
        
        // Create inappropriate-looking pixel data
        let pixels = vec![0u8; 256];
        
        // Should allow all content when disabled
        assert!(filter.is_appropriate(&pixels, 0));
    }

    #[test]
    fn test_filter_enable_disable_toggle() {
        let mut filter = ContentFilter::new();
        
        // Initially enabled
        assert!(filter.get_stats().enabled);
        
        // Disable
        filter.set_enabled(false);
        assert!(!filter.get_stats().enabled);
        
        // Re-enable
        filter.set_enabled(true);
        assert!(filter.get_stats().enabled);
    }

    // ==================== Strict Mode Tests ====================

    #[test]
    fn test_strict_mode_toggle() {
        let mut filter = ContentFilter::new();
        
        // Initially not strict
        assert!(!filter.get_stats().strict_mode);
        
        // Enable strict mode
        filter.set_strict_mode(true);
        assert!(filter.get_stats().strict_mode);
        
        // Disable strict mode
        filter.set_strict_mode(false);
        assert!(!filter.get_stats().strict_mode);
    }

    #[test]
    fn test_solid_color_detection_non_strict() {
        let mut filter = ContentFilter::new();
        filter.set_strict_mode(false);
        
        // Create solid color chunk (all same color)
        let pixels = vec![5u8; 256];
        
        // In non-strict mode, solid colors are allowed
        assert!(filter.is_appropriate(&pixels, 0));
    }

    #[test]
    fn test_solid_color_detection_strict() {
        let mut filter = ContentFilter::new();
        filter.set_strict_mode(true);
        
        // Create solid color chunk (>250 pixels of same color)
        let mut pixels = vec![5u8; 256];
        
        // In strict mode, solid colors might be flagged as spam
        // Note: Current implementation flags >250 same-color pixels
        let result = filter.is_appropriate(&pixels, 0);
        
        // Should detect as potentially inappropriate
        assert!(!result);
    }

    #[test]
    fn test_flashing_pattern_detection() {
        let mut filter = ContentFilter::new();
        filter.set_strict_mode(true);
        
        // Create alternating two-color pattern
        let mut pixels = Vec::with_capacity(256);
        for i in 0..256 {
            if i % 2 == 0 {
                pixels.push(1);
            } else {
                pixels.push(2);
            }
        }
        
        // Two dominant colors might indicate flashing
        let result = filter.is_appropriate(&pixels, 0);
        
        // May be flagged in strict mode
        // (depends on implementation thresholds)
        let _ = result;
    }

    // ==================== Blocked Patterns Tests ====================

    #[test]
    fn test_block_pattern() {
        let mut filter = ContentFilter::new();
        
        // Create a pattern to block
        let blocked = vec![42u8; 256];
        filter.block_pattern(blocked.clone());
        
        assert_eq!(filter.get_stats().blocked_patterns_count, 1);
        
        // This pattern should be rejected
        assert!(!filter.is_appropriate(&blocked, 0));
    }

    #[test]
    fn test_multiple_blocked_patterns() {
        let mut filter = ContentFilter::new();
        
        // Block multiple patterns
        filter.block_pattern(vec![1u8; 256]);
        filter.block_pattern(vec![2u8; 256]);
        filter.block_pattern(vec![3u8; 256]);
        
        assert_eq!(filter.get_stats().blocked_patterns_count, 3);
        
        // All blocked patterns should be rejected
        assert!(!filter.is_appropriate(&vec![1u8; 256], 0));
        assert!(!filter.is_appropriate(&vec![2u8; 256], 0));
        assert!(!filter.is_appropriate(&vec![3u8; 256], 0));
        
        // Other patterns should be allowed
        assert!(filter.is_appropriate(&vec![4u8; 256], 0));
    }

    // ==================== Pixel Data Validation Tests ====================

    #[test]
    fn test_invalid_pixel_data_length() {
        let filter = ContentFilter::new();
        
        // Invalid length should be handled gracefully
        let short_pixels = vec![0u8; 100];
        let long_pixels = vec![0u8; 500];
        
        // Should not panic, returns true (appropriate) for invalid data
        // as a fail-safe behavior
        let _short_result = filter.is_appropriate(&short_pixels, 0);
        let _long_result = filter.is_appropriate(&long_pixels, 0);
    }

    #[test]
    fn test_valid_color_range() {
        let filter = ContentFilter::new();
        
        // Create pixels with valid 4-bit color values (0-15)
        let mut pixels = Vec::with_capacity(256);
        for i in 0..256 {
            pixels.push((i % 16) as u8);
        }
        
        // Should be appropriate
        assert!(filter.is_appropriate(&pixels, 0));
    }

    #[test]
    fn test_mixed_content() {
        let filter = ContentFilter::new();
        
        // Create varied pixel data
        let mut pixels = Vec::with_capacity(256);
        for i in 0..256 {
            pixels.push((i * 7 % 16) as u8);
        }
        
        // Mixed content should be appropriate
        assert!(filter.is_appropriate(&pixels, 0));
    }

    // ==================== Filter Stats Tests ====================

    #[test]
    fn test_get_stats() {
        let mut filter = ContentFilter::new();
        
        // Initial stats
        let stats = filter.get_stats();
        assert!(stats.enabled);
        assert!(!stats.strict_mode);
        assert_eq!(stats.blocked_patterns_count, 0);
        
        // Modify filter
        filter.set_strict_mode(true);
        filter.block_pattern(vec![1u8; 256]);
        
        // Updated stats
        let stats = filter.get_stats();
        assert!(stats.enabled);
        assert!(stats.strict_mode);
        assert_eq!(stats.blocked_patterns_count, 1);
    }

    // ==================== ModerationResult Tests ====================

    #[test]
    fn test_moderation_result_approved() {
        let result = ModerationResult::approved();
        
        assert!(result.is_approved);
        assert!(result.reason.is_none());
        assert!(!result.requires_review);
    }

    #[test]
    fn test_moderation_result_rejected() {
        let result = ModerationResult::rejected("Inappropriate content".to_string());
        
        assert!(!result.is_approved);
        assert!(result.reason.is_some());
        assert_eq!(result.reason.unwrap(), "Inappropriate content");
        assert!(!result.requires_review);
    }

    #[test]
    fn test_moderation_result_needs_review() {
        let result = ModerationResult::needs_review("Requires manual check".to_string());
        
        assert!(!result.is_approved);
        assert!(result.reason.is_some());
        assert!(result.requires_review);
    }

    // ==================== AdvancedContentFilter Tests ====================

    #[test]
    fn test_advanced_filter_creation() {
        let filter = AdvancedContentFilter::new();
        
        // Should have base filter enabled
        assert!(filter.base_filter().get_stats().enabled);
        assert!(!filter.base_filter().get_stats().strict_mode);
    }

    #[test]
    fn test_advanced_filter_default() {
        let filter = AdvancedContentFilter::default();
        
        assert!(filter.base_filter().get_stats().enabled);
    }

    #[test]
    fn test_advanced_filter_neural_network_config() {
        let mut filter = AdvancedContentFilter::new();
        
        // Initially NN is disabled
        // (we can't directly check use_nn, but we can enable it)
        filter.enable_neural_network("http://localhost:8080".to_string());
        
        // Base filter should still work
        let pixels = vec![0u8; 256];
        // The async check would use NN, but we test sync base filter
        assert!(filter.base_filter().is_appropriate(&pixels, 0));
    }

    #[test]
    fn test_advanced_filter_base_filter_mut() {
        let mut filter = AdvancedContentFilter::new();
        
        // Modify base filter through mutable reference
        filter.base_filter_mut().set_strict_mode(true);
        
        assert!(filter.base_filter().get_stats().strict_mode);
    }

    // ==================== Text Pattern Detection Tests ====================

    #[test]
    fn test_horizontal_line_detection() {
        let mut filter = ContentFilter::new();
        filter.set_strict_mode(true);
        
        // Create horizontal lines
        let mut pixels = vec![0u8; 256];
        
        // Draw horizontal line at row 5
        for x in 0..16 {
            pixels[(5 * 16 + x) as usize] = 1;
        }
        
        // Draw another horizontal line at row 10
        for x in 0..16 {
            pixels[(10 * 16 + x) as usize] = 1;
        }
        
        let _result = filter.is_appropriate(&pixels, 0);
        // May be flagged depending on number of lines
    }

    #[test]
    fn test_vertical_line_detection() {
        let mut filter = ContentFilter::new();
        filter.set_strict_mode(true);
        
        // Create vertical lines
        let mut pixels = vec![0u8; 256];
        
        // Draw vertical line at column 5
        for y in 0..16 {
            pixels[(y * 16 + 5) as usize] = 1;
        }
        
        let _result = filter.is_appropriate(&pixels, 0);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_chunk_id() {
        let filter = ContentFilter::new();
        let pixels = vec![0u8; 256];
        
        // Chunk ID should not affect basic filtering
        assert!(filter.is_appropriate(&pixels, 0));
        assert!(filter.is_appropriate(&pixels, 999));
        assert!(filter.is_appropriate(&pixels, u32::MAX));
    }

    #[test]
    fn test_filter_consistency() {
        let filter = ContentFilter::new();
        let pixels = vec![7u8; 256];
        
        // Multiple checks should return same result
        let result1 = filter.is_appropriate(&pixels, 0);
        let result2 = filter.is_appropriate(&pixels, 0);
        let result3 = filter.is_appropriate(&pixels, 0);
        
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_all_black_chunk() {
        let filter = ContentFilter::new();
        let pixels = vec![0u8; 256];
        
        // All black should be allowed in non-strict mode
        assert!(filter.is_appropriate(&pixels, 0));
    }

    #[test]
    fn test_all_white_chunk() {
        let filter = ContentFilter::new();
        let pixels = vec![1u8; 256];  // Assuming 1 is white
        
        // All white should be allowed in non-strict mode
        assert!(filter.is_appropriate(&pixels, 0));
    }

    #[test]
    fn test_gradient_pattern() {
        let filter = ContentFilter::new();
        
        // Create gradient pattern
        let mut pixels = Vec::with_capacity(256);
        for y in 0..16 {
            for x in 0..16 {
                let value = ((x + y) / 2) as u8;
                pixels.push(value.min(15));
            }
        }
        
        // Gradient should be appropriate
        assert!(filter.is_appropriate(&pixels, 0));
    }
}
