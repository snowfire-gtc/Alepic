use eframe::egui;
use crate::mode::OperationMode;

/// Billboard mode: Read-only display without UI interactions
/// Used for displaying the canvas on city streets or public displays
pub struct BillboardMode {
    enabled: bool,
    /// Auto-refresh interval in seconds
    refresh_interval: u64,
    /// Last refresh time (simulation)
    last_refresh: u64,
    /// Show Alepe animation
    show_alepe: bool,
    /// Show chunk ownership info (optional overlay)
    show_info_overlay: bool,
}

impl BillboardMode {
    pub fn new() -> Self {
        Self {
            enabled: false,
            refresh_interval: 60, // Refresh every 60 seconds
            last_refresh: 0,
            show_alepe: true,
            show_info_overlay: false,
        }
    }

    /// Enable billboard mode
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable billboard mode
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if billboard mode is active
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set refresh interval
    pub fn set_refresh_interval(&mut self, seconds: u64) {
        self.refresh_interval = seconds;
    }

    /// Enable/disable Alepe display
    pub fn set_show_alepe(&mut self, show: bool) {
        self.show_alepe = show;
    }

    /// Enable/disable info overlay
    pub fn set_show_info_overlay(&mut self, show: bool) {
        self.show_info_overlay = show;
    }

    /// Check if refresh is needed
    pub fn needs_refresh(&self, current_time: u64) -> bool {
        current_time >= self.last_refresh + self.refresh_interval
    }

    /// Mark as refreshed
    pub fn mark_refreshed(&mut self, current_time: u64) {
        self.last_refresh = current_time;
    }

    /// Render billboard view (read-only, no UI elements)
    pub fn render_billboard(
        &self,
        ui: &egui::Ui,
        texture: &egui::TextureHandle,
        viewport: &crate::app::Viewport,
        available_rect: egui::Rect,
    ) {
        // Calculate visible area based on viewport
        let zoom = viewport.zoom;
        let offset = viewport.offset;

        // Draw the full canvas texture scaled to fit
        let canvas_size = egui::vec2(4096.0, 2160.0); // UHD resolution
        let scaled_size = canvas_size * zoom;

        // Center the canvas in the available area
        let pos = egui::pos2(
            available_rect.center().x - scaled_size.x / 2.0 + offset.x,
            available_rect.center().y - scaled_size.y / 2.0 + offset.y,
        );

        let image_rect = egui::Rect::from_min_size(pos, scaled_size);

        // Draw the canvas image
        ui.painter().image(
            texture.id(),
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        // Optional: Show info overlay
        if self.show_info_overlay {
            self.render_info_overlay(ui, available_rect);
        }
    }

    /// Render informational overlay (optional)
    fn render_info_overlay(&self, ui: &egui::Ui, rect: egui::Rect) {
        // Top-left corner: Project name and timestamp
        egui::Area::new("billboard_info")
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(20.0, 20.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::from_black_alpha(150))
                    .rounding(5.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Alepic").color(egui::Color32::GREEN));
                        ui.label("Collaborative Canvas");
                        ui.label(format!("Updated: {}", self.get_timestamp()));
                    });
            });

        // Bottom-right: Alepe status (if enabled)
        if self.show_alepe {
            egui::Area::new("billboard_alepe_status")
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_black_alpha(150))
                        .rounding(5.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.label("🐸 Alepe");
                            ui.label("Next jump: soon...");
                        });
                });
        }
    }

    /// Get formatted timestamp
    fn get_timestamp(&self) -> String {
        // In real implementation, use actual time
        "Just now".to_string()
    }

    /// Handle input in billboard mode (limited interactions)
    pub fn handle_billboard_input(
        &mut self,
        ctx: &egui::Context,
        viewport: &mut crate::app::Viewport,
    ) {
        // Only allow zoom and pan, no drawing or transactions
        let input = ctx.input(|i| i.clone());

        // Zoom: Mouse Wheel
        if let Some(delta) = input.scroll_delta.y {
            viewport.zoom *= 1.0 + (delta * 0.001);
            viewport.zoom = viewport.zoom.clamp(0.1, 5.0); // More zoom range for viewing
        }

        // Pan: Left mouse drag
        if input.pointer.button_down(egui::PointerButton::Primary) {
            if let Some(delta) = input.pointer.delta() {
                viewport.offset += delta;
            }
        }

        // Keyboard navigation
        if input.key_pressed(egui::Key::ArrowUp) {
            viewport.offset.y += 50.0;
        }
        if input.key_pressed(egui::Key::ArrowDown) {
            viewport.offset.y -= 50.0;
        }
        if input.key_pressed(egui::Key::ArrowLeft) {
            viewport.offset.x += 50.0;
        }
        if input.key_pressed(egui::Key::ArrowRight) {
            viewport.offset.x -= 50.0;
        }

        // Reset view with 'R' key
        if input.key_pressed(egui::Key::R) {
            viewport.zoom = 1.0;
            viewport.offset = egui::Vec2::ZERO;
        }
    }
}

impl Default for BillboardMode {
    fn default() -> Self {
        Self::new()
    }
}

/// Display configuration for different screen types
#[derive(Debug, Clone)]
pub enum DisplayType {
    /// Standard monitor (16:9)
    Monitor,
    /// Vertical display (9:16)
    Vertical,
    /// Ultrawide (21:9)
    Ultrawide,
    /// Custom aspect ratio
    Custom(f32, f32),
}

impl DisplayType {
    pub fn get_aspect_ratio(&self) -> f32 {
        match self {
            DisplayType::Monitor => 16.0 / 9.0,
            DisplayType::Vertical => 9.0 / 16.0,
            DisplayType::Ultrawide => 21.0 / 9.0,
            DisplayType::Custom(w, h) => *w / *h,
        }
    }

    pub fn get_optimal_zoom(&self, screen_width: f32, screen_height: f32) -> f32 {
        let canvas_width = 4096.0;
        let canvas_height = 2160.0;

        let zoom_w = screen_width / canvas_width;
        let zoom_h = screen_height / canvas_height;

        zoom_w.min(zoom_h)
    }
}

/// Public display manager for billboard installations
pub struct PublicDisplayManager {
    billboard_mode: BillboardMode,
    display_type: DisplayType,
    /// Screen dimensions in pixels
    screen_width: f32,
    screen_height: f32,
    /// Auto-cycle through predefined views
    auto_cycle: bool,
    cycle_interval: u64,
    current_view_index: usize,
}

impl PublicDisplayManager {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let display_type = Self::detect_display_type(screen_width, screen_height);

        Self {
            billboard_mode: BillboardMode::new(),
            display_type,
            screen_width,
            screen_height,
            auto_cycle: false,
            cycle_interval: 30, // 30 seconds per view
            current_view_index: 0,
        }
    }

    /// Detect display type from dimensions
    fn detect_display_type(width: f32, height: f32) -> DisplayType {
        let ratio = width / height;
        
        if ratio > 2.0 {
            DisplayType::Ultrawide
        } else if ratio < 1.0 {
            DisplayType::Vertical
        } else {
            DisplayType::Monitor
        }
    }

    /// Enable auto-cycling through views
    pub fn enable_auto_cycle(&mut self, interval_seconds: u64) {
        self.auto_cycle = true;
        self.cycle_interval = interval_seconds;
    }

    /// Get optimal initial viewport for this display
    pub fn get_initial_viewport(&self) -> crate::app::Viewport {
        let zoom = self.display_type.get_optimal_zoom(self.screen_width, self.screen_height);
        
        crate::app::Viewport {
            zoom,
            offset: egui::Vec2::ZERO,
        }
    }

    /// Get the billboard mode reference
    pub fn billboard_mode(&self) -> &BillboardMode {
        &self.billboard_mode
    }

    /// Get mutable billboard mode reference
    pub fn billboard_mode_mut(&mut self) -> &mut BillboardMode {
        &mut self.billboard_mode
    }

    /// Update auto-cycle (call each frame)
    pub fn update_cycle(&mut self, delta_time: f32) {
        if !self.auto_cycle {
            return;
        }

        // In real implementation, track elapsed time and cycle views
        // Views could be: full canvas, zoomed on Alepe, recent activity, etc.
    }
}

impl Default for PublicDisplayManager {
    fn default() -> Self {
        Self::new(1920.0, 1080.0) // Full HD default
    }
}
