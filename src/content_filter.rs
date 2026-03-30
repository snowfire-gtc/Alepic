use std::collections::HashSet;

/// Content filter for inappropriate content management
/// Client Providers can use this to filter content based on jurisdiction rules
pub struct ContentFilter {
    /// Enable/disable filtering
    enabled: bool,
    /// Strict mode: blocks more content
    strict_mode: bool,
    /// Blocked color patterns (simplified heuristic)
    blocked_patterns: HashSet<Vec<u8>>,
}

impl ContentFilter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            strict_mode: false,
            blocked_patterns: HashSet::new(),
        }
    }

    /// Enable or disable content filtering
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set strict mode for more aggressive filtering
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Check if pixel data contains inappropriate content
    /// Returns true if content is appropriate, false otherwise
    pub fn is_appropriate(&self, pixels: &[u8], chunk_id: u32) -> bool {
        if !self.enabled {
            return true; // Filtering disabled
        }

        // Check for blocked patterns
        if self.blocked_patterns.contains(pixels) {
            return false;
        }

        // Heuristic checks
        if self.detect_inappropriate_pattern(pixels) {
            return false;
        }

        true
    }

    /// Detect inappropriate patterns using simple heuristics
    /// In production, this would use neural networks as mentioned in README
    fn detect_inappropriate_pattern(&self, pixels: &[u8]) -> bool {
        if pixels.len() != 256 {
            return false; // Invalid chunk size
        }

        // Count color distribution
        let mut color_counts = [0u32; 16];
        for &pixel in pixels {
            if pixel < 16 {
                color_counts[pixel as usize] += 1;
            }
        }

        // Check for solid color chunks (potential spam)
        let max_count = *color_counts.iter().max().unwrap_or(&0);
        if max_count > 250 {
            // More than 250 pixels of same color might be spam
            if self.strict_mode {
                return true;
            }
        }

        // Check for flashing patterns (rapid alternation)
        // Simplified: check if only 2 colors dominate
        let dominant_colors: Vec<_> = color_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count > 100)
            .collect();
        
        if dominant_colors.len() == 2 && self.strict_mode {
            // Two dominant colors might indicate flashing
            return true;
        }

        // Check for specific color combinations that might be inappropriate
        // (e.g., certain symbols or text-like patterns)
        if self.detect_text_like_pattern(pixels) && self.strict_mode {
            return true;
        }

        false
    }

    /// Detect text-like patterns (simplified)
    fn detect_text_like_pattern(&self, pixels: &[u8]) -> bool {
        // Look for horizontal or vertical lines that might form text
        let mut h_lines = 0;
        let mut v_lines = 0;

        // Check horizontal lines
        for y in 0..16 {
            let mut line_color: Option<u8> = None;
            let mut line_length = 0;
            for x in 0..16 {
                let idx = (y * 16 + x) as usize;
                if Some(pixels[idx]) == line_color {
                    line_length += 1;
                } else {
                    if line_length >= 10 {
                        h_lines += 1;
                    }
                    line_color = Some(pixels[idx]);
                    line_length = 1;
                }
            }
            if line_length >= 10 {
                h_lines += 1;
            }
        }

        // Check vertical lines
        for x in 0..16 {
            let mut line_color: Option<u8> = None;
            let mut line_length = 0;
            for y in 0..16 {
                let idx = (y * 16 + x) as usize;
                if Some(pixels[idx]) == line_color {
                    line_length += 1;
                } else {
                    if line_length >= 10 {
                        v_lines += 1;
                    }
                    line_color = Some(pixels[idx]);
                    line_length = 1;
                }
            }
            if line_length >= 10 {
                v_lines += 1;
            }
        }

        // Multiple long lines might indicate text
        h_lines + v_lines >= 8
    }

    /// Add a pattern to the blocked list
    pub fn block_pattern(&mut self, pattern: Vec<u8>) {
        self.blocked_patterns.insert(pattern);
    }

    /// Get filter statistics
    pub fn get_stats(&self) -> FilterStats {
        FilterStats {
            enabled: self.enabled,
            strict_mode: self.strict_mode,
            blocked_patterns_count: self.blocked_patterns.len(),
        }
    }
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the content filter
#[derive(Debug, Clone)]
pub struct FilterStats {
    pub enabled: bool,
    pub strict_mode: bool,
    pub blocked_patterns_count: usize,
}

/// Content moderation result
#[derive(Debug, Clone)]
pub struct ModerationResult {
    pub is_approved: bool,
    pub reason: Option<String>,
    pub requires_review: bool,
}

impl ModerationResult {
    pub fn approved() -> Self {
        Self {
            is_approved: true,
            reason: None,
            requires_review: false,
        }
    }

    pub fn rejected(reason: String) -> Self {
        Self {
            is_approved: false,
            reason: Some(reason),
            requires_review: false,
        }
    }

    pub fn needs_review(reason: String) -> Self {
        Self {
            is_approved: false,
            reason: Some(reason),
            requires_review: true,
        }
    }
}

/// Advanced content filter with neural network integration support
pub struct AdvancedContentFilter {
    base_filter: ContentFilter,
    /// Use external neural network service
    use_nn: bool,
    /// NN service URL (if applicable)
    nn_service_url: Option<String>,
}

impl AdvancedContentFilter {
    pub fn new() -> Self {
        Self {
            base_filter: ContentFilter::new(),
            use_nn: false,
            nn_service_url: None,
        }
    }

    /// Enable neural network-based filtering
    pub fn enable_neural_network(&mut self, service_url: String) {
        self.use_nn = true;
        self.nn_service_url = Some(service_url);
    }

    /// Check content with advanced filtering
    pub async fn check_content(&self, pixels: &[u8], chunk_id: u32) -> ModerationResult {
        // First, run basic filter
        if !self.base_filter.is_appropriate(pixels, chunk_id) {
            return ModerationResult::rejected("Failed basic content filter".to_string());
        }

        // If neural network is enabled, use it for additional checking
        if self.use_nn {
            match self.check_with_neural_network(pixels).await {
                Ok(is_appropriate) => {
                    if !is_appropriate {
                        return ModerationResult::needs_review(
                            "Flagged by neural network - requires manual review".to_string()
                        );
                    }
                }
                Err(e) => {
                    // NN service unavailable, fall back to basic filter
                    eprintln!("Neural network service error: {}", e);
                }
            }
        }

        ModerationResult::approved()
    }

    /// Check content using neural network service
    async fn check_with_neural_network(&self, pixels: &[u8]) -> Result<bool, String> {
        if let Some(url) = &self.nn_service_url {
            // In real implementation, send pixels to NN service
            // For now, return mock result
            Ok(true)
        } else {
            Err("No neural network service configured".to_string())
        }
    }

    /// Get the base filter
    pub fn base_filter(&self) -> &ContentFilter {
        &self.base_filter
    }

    /// Get mutable base filter
    pub fn base_filter_mut(&mut self) -> &mut ContentFilter {
        &mut self.base_filter
    }
}

impl Default for AdvancedContentFilter {
    fn default() -> Self {
        Self::new()
    }
}
