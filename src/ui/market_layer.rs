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

//! Market Layer UI module
//! 
//! Provides the market interface for buying, selling, and auction interactions.

use eframe::egui;

/// Market layer configuration
pub struct MarketLayerConfig {
    pub show_prices: bool,
    pub show_auction_highlights: bool,
    pub show_ownership_info: bool,
}

impl Default for MarketLayerConfig {
    fn default() -> Self {
        Self {
            show_prices: true,
            show_auction_highlights: true,
            show_ownership_info: true,
        }
    }
}

/// Render the market layer UI elements
/// 
/// # Arguments
/// * `ctx` - The egui context
/// * `on_wallet_connect` - Callback function when user clicks wallet connect button
pub fn render_market_layer(ctx: &egui::Context, on_wallet_connect: &mut dyn FnMut()) {
    // Wallet Connect Button (Bottom Right)
    egui::Area::new("wallet_button")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
        .show(ctx, |ui| {
            if ui.button("👛 Connect Wallet").clicked() {
                on_wallet_connect();
            }
        });
}

/// Render chunk price label above a chunk
/// 
/// # Arguments
/// * `ui` - The UI context
/// * `chunk_rect` - Screen rectangle of the chunk
/// * `price_alph` - Price in ALPH tokens
/// * `is_auction` - Whether this is an auction chunk
pub fn render_chunk_price_label(
    ui: &egui::Ui,
    chunk_rect: egui::Rect,
    price_alph: f64,
    is_auction: bool,
) {
    let label_pos = egui::pos2(chunk_rect.center().x, chunk_rect.min.y - 5.0);
    
    let bg_color = if is_auction {
        egui::Color32::from_rgb(255, 140, 0) // Dark orange for auction
    } else {
        egui::Color32::from_black_alpha(180)
    };
    
    egui::Frame::none()
        .fill(bg_color)
        .rounding(3.0)
        .inner_margin(egui::vec2(4.0, 2.0))
        .show_at(ui.ctx(), egui::Rect::from_min_size(label_pos, egui::vec2(0.0, 0.0)), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2} ALPH", price_alph))
                    .color(egui::Color32::WHITE)
                    .size(10.0)
            );
        });
}

/// Render auction chunk highlight overlay
/// 
/// # Arguments
/// * `painter` - The painter to draw with
/// * `chunk_rect` - Screen rectangle of the chunk
pub fn render_auction_highlight(painter: &egui::Painter, chunk_rect: egui::Rect) {
    // Animated border effect for auction chunks
    let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 140, 0));
    painter.rect_stroke(chunk_rect, 2.0, stroke);
    
    // Subtle glow effect
    let glow_rect = chunk_rect.expand(3.0);
    painter.rect_stroke(glow_rect, 3.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 140, 0).linear_multiply(0.3)));
}

/// Render ownership indicator for owned chunks
/// 
/// # Arguments
/// * `painter` - The painter to draw with
/// * `chunk_rect` - Screen rectangle of the chunk
/// * `is_owned_by_user` - Whether the current user owns this chunk
pub fn render_ownership_indicator(
    painter: &egui::Painter,
    chunk_rect: egui::Rect,
    is_owned_by_user: bool,
) {
    let color = if is_owned_by_user {
        egui::Color32::from_rgb(0, 200, 0) // Green for owned
    } else {
        egui::Color32::from_rgb(200, 0, 0) // Red for other owners
    };
    
    // Small corner indicator
    let indicator_size = 8.0;
    let indicator_rect = egui::Rect::from_min_size(
        egui::pos2(chunk_rect.min.x, chunk_rect.min.y),
        egui::vec2(indicator_size, indicator_size),
    );
    
    painter.rect_filled(indicator_rect, 0.0, color);
}