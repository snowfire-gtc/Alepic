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

//! Color Layer UI module
//! 
//! Provides the color palette and drawing tools for the Color Layer view.

use eframe::egui;
use crate::canvas::chunk::Palette;

/// Render the color layer UI elements
/// 
/// # Arguments
/// * `ctx` - The egui context
/// * `selected_color` - Mutable reference to the currently selected color index
/// * `on_submit` - Callback function when user clicks submit
pub fn render_color_layer(ctx: &egui::Context, selected_color: &mut u8, on_submit: &mut dyn FnMut()) {
    // 1. Color Palette (Bottom Center)
    // Two rows: bright colors (0-7), dark colors (8-15)
    egui::Area::new("color_palette")
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -60.0))
        .show(ctx, |ui| {
            egui::Frame::panel()
                .fill(egui::Color32::from_black_alpha(200))
                .rounding(10.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // First row: bright colors (0-7)
                        ui.horizontal(|ui| {
                            for i in 0..8u8 {
                                let color = Palette::get_color(i);
                                let btn_size = egui::vec2(30.0, 30.0);
                                let response = ui.add(
                                    egui::Button::new("")
                                        .fill(color)
                                        .min_size(btn_size)
                                );
                                if response.clicked() {
                                    *selected_color = i;
                                }
                                // Highlight selected color
                                if i == *selected_color {
                                    response.highlight();
                                }
                            }
                        });
                        // Second row: dark colors (8-15)
                        ui.horizontal(|ui| {
                            for i in 8..16u8 {
                                let color = Palette::get_color(i);
                                let btn_size = egui::vec2(30.0, 30.0);
                                let response = ui.add(
                                    egui::Button::new("")
                                        .fill(color)
                                        .min_size(btn_size)
                                );
                                if response.clicked() {
                                    *selected_color = i;
                                }
                                // Highlight selected color
                                if i == *selected_color {
                                    response.highlight();
                                }
                            }
                        });
                    });
                });
        });

    // 2. Submit Button (Bottom Right)
    egui::Area::new("submit_button")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
        .show(ctx, |ui| {
            if ui.button("💾 Submit").clicked() {
                on_submit();
            }
        });
}