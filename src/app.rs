use eframe::egui;
use std::collections::HashMap;
use crate::canvas::chunk::{Chunk, CHUNK_SIZE};
use crate::rendering::texture_mgr::TextureManager;
use crate::game::alepe::Alepe;
use crate::blockchain::wallet::WalletManager;
use crate::blockchain::contract::{AlepicContract, ChunkInfo, AuctionInfo};
use crate::blockchain::transactions::{TransactionBuilder, TransactionData, TransactionStatus};
use crate::blockchain::manager::{BlockchainManager, TransactionError};
use crate::mode::OperationMode;
use crate::content_filter::{AdvancedContentFilter, ModerationResult};
use crate::billboard::BillboardMode;

pub struct AlepicApp {
    chunks: HashMap<u32, Chunk>,
    chunk_infos: HashMap<u32, ChunkInfo>,
    texture_mgr: TextureManager,
    viewport: Viewport,
    current_layer: LayerType,
    alepe: Alepe,
    wallet: WalletManager,
    blockchain: BlockchainManager,
    content_filter: AdvancedContentFilter,
    billboard_mode: BillboardMode,
    selected_color: u8,
    pending_changes: Vec<PixelChange>,
    
    // Dialog states
    show_wallet_dialog: bool,
    show_buy_dialog: Option<u32>,
    show_sell_dialog: Option<u32>,
    show_stake_dialog: Option<u32>,
    show_submit_dialog: bool,
    show_mode_dialog: bool,
    
    // Transaction state
    pending_transaction: Option<TransactionData>,
    transaction_status: TransactionStatus,
    last_transaction_error: Option<TransactionError>,
    
    // Current block number (for Alepe jumps)
    current_block: u64,
}

enum LayerType {
    Color,
    Market,
}

pub struct Viewport {
    pub zoom: f32,
    pub offset: egui::Vec2,
}

struct PixelChange {
    chunk_id: u32,
    x: u16,
    y: u16,
    color: u8,
}

impl eframe::App for AlepicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Обработка ввода (Zoom/Pan)
        self.handle_input(ctx);

        // 2. Отрисовка Холста (Центральная панель)
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_canvas(ui);
        });

        // 3. Отрисовка UI Слоев
        self.render_layer_ui(ctx);

        // 4. Проверка событий Alepe
        if let Some(client) = self.wallet.get_client() {
            // В реальной реализации получаем номер блока из блокчейна
            // self.current_block = client.get_current_block().await.unwrap_or(self.current_block).block_number;
        }
        self.alepe.check_jump(self.current_block);

        // 5. Диалоги
        self.render_dialogs(ctx);
    }
}

impl AlepicApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Default to simulation mode
        let mut blockchain = BlockchainManager::new(OperationMode::Simulation);
        
        Self {
            chunks: HashMap::new(),
            chunk_infos: HashMap::new(),
            texture_mgr: TextureManager::new(&cc.egui_ctx),
            viewport: Viewport {
                zoom: 1.0,
                offset: egui::Vec2::ZERO,
            },
            current_layer: LayerType::Color,
            alepe: Alepe::new(),
            wallet: WalletManager::new(),
            blockchain,
            content_filter: AdvancedContentFilter::new(),
            billboard_mode: BillboardMode::new(),
            selected_color: 1,
            pending_changes: Vec::new(),
            show_wallet_dialog: false,
            show_buy_dialog: None,
            show_sell_dialog: None,
            show_stake_dialog: None,
            show_submit_dialog: false,
            show_mode_dialog: false,
            pending_transaction: None,
            transaction_status: TransactionStatus::Pending,
            last_transaction_error: None,
            current_block: 0,
        }
    }

    /// Initialize blockchain connection
    pub fn init_blockchain(&mut self, node_url: String, contract_address: String) {
        self.wallet.init(node_url.clone(), contract_address.clone());
        self.blockchain.init(node_url, contract_address);
    }
    
    /// Toggle between Real and Simulation mode
    pub fn toggle_mode(&mut self) {
        let current = self.blockchain.get_mode();
        let new_mode = match current {
            OperationMode::Real => OperationMode::Simulation,
            OperationMode::Simulation => OperationMode::Real,
        };
        self.blockchain.set_mode(new_mode);
    }
    
    /// Enable billboard mode
    pub fn enable_billboard_mode(&mut self) {
        self.billboard_mode.enable();
    }
    
    /// Disable billboard mode
    pub fn disable_billboard_mode(&mut self) {
        self.billboard_mode.disable();
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        // Zoom: Mouse Wheel
        if let Some(delta) = input.scroll_delta.y {
            self.viewport.zoom *= 1.0 + (delta * 0.001);
            self.viewport.zoom = self.viewport.zoom.clamp(0.5, 10.0);
        }

        // Pan: Left Mouse Drag
        if input.pointer.button_down(egui::PointerButton::Primary) {
            if let Some(delta) = input.pointer.delta() {
                // Только если не кликаем по UI элементам
                self.viewport.offset += delta;
            }
        }
    }

    fn render_canvas(&mut self, ui: &egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();

        // Расчет видимых чанков на основе вьюпорта
        let visible_chunks = self.calculate_visible_chunks(available_rect);

        for chunk_id in &visible_chunks {
            if let Some(chunk) = self.chunks.get(chunk_id) {
                // Обновление текстуры при необходимости
                self.texture_mgr.update_chunk(chunk);

                // Отрисовка изображения чанка
                if let Some(texture) = self.texture_mgr.get_texture(*chunk_id) {
                    let pos = self.get_chunk_screen_pos(chunk, available_rect);
                    let size = egui::vec2(16.0, 16.0) * self.viewport.zoom;

                    let mut image = egui::Image::new(texture, size);

                    // Подсветка при наведении
                    if ui.is_rect_hovered(pos) {
                        image = image.tint(egui::Color32::from_white_alpha(100));
                    }

                    ui.put(pos, image);
                }
            }
        }

        // Очистка невидимых текстур
        self.texture_mgr.cleanup(&visible_chunks);

        // Отрисовка Alepe (если виден)
        self.alepe.render(ui, &self.viewport, available_rect);
    }

    fn render_layer_ui(&mut self, ctx: &egui::Context) {
        match self.current_layer {
            LayerType::Color => self.render_color_layer(ctx),
            LayerType::Market => self.render_market_layer(ctx),
        }
    }

    fn calculate_visible_chunks(&self, rect: egui::Rect) -> Vec<u32> {
        // Заглушка для расчета видимых чанков
        // В реальной реализации нужно учитывать zoom, offset и размер экрана
        vec![0, 1, 2, 3, 4, 5] // Пример чанков
    }

    fn get_chunk_screen_pos(&self, chunk: &Chunk, rect: egui::Rect) -> egui::Rect {
        // Заглушка для расчета позиции чанка на экране
        let base_x = rect.min.x + self.viewport.offset.x;
        let base_y = rect.min.y + self.viewport.offset.y;
        let grid_x = chunk.grid_x as f32 * 16.0 * self.viewport.zoom;
        let grid_y = chunk.grid_y as f32 * 16.0 * self.viewport.zoom;
        
        egui::Rect::from_min_size(
            egui::pos2(base_x + grid_x, base_y + grid_y),
            egui::vec2(16.0 * self.viewport.zoom, 16.0 * self.viewport.zoom),
        )
    }

    fn render_color_layer(&mut self, ctx: &egui::Context) {
        // Палитра цветов (Bottom Center)
        egui::Area::new("color_palette")
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -60.0))
            .show(ctx, |ui| {
                egui::Frame::panel()
                    .fill(egui::Color32::from_black_alpha(200))
                    .rounding(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..16u8 {
                                let color = crate::canvas::chunk::Palette::get_color(i);
                                let btn_size = egui::vec2(25.0, 25.0);
                                let response = ui.add(
                                    egui::Button::new("")
                                        .fill(color)
                                        .min_size(btn_size)
                                );
                                if response.clicked() {
                                    self.selected_color = i;
                                }
                                // Подсветка выбранного цвета
                                if i == self.selected_color {
                                    response.highlight();
                                }
                            }
                        });
                    });
            });

        // Кнопка переключения слоя (Bottom Left)
        egui::Area::new("layer_switch")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(20.0, -20.0))
            .show(ctx, |ui| {
                if ui.button("🎨 Layers").clicked() {
                    self.current_layer = LayerType::Market;
                }
            });

        // Кнопка Submit (Bottom Right)
        egui::Area::new("submit_button")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
            .show(ctx, |ui| {
                if ui.button("💾 Submit").clicked() {
                    self.show_submit_dialog = true;
                }
            });
    }

    fn render_market_layer(&mut self, ctx: &egui::Context) {
        // Кнопка переключения слоя (Bottom Left)
        egui::Area::new("layer_switch")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(20.0, -20.0))
            .show(ctx, |ui| {
                if ui.button("🏠 Back to Canvas").clicked() {
                    self.current_layer = LayerType::Color;
                }
            });

        // Кнопка Wallet Connect (Bottom Right)
        egui::Area::new("wallet_button")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0))
            .show(ctx, |ui| {
                if ui.button("👛 Connect Wallet").clicked() {
                    self.show_wallet_dialog = true;
                }
            });
    }

    /// Render all dialogs
    fn render_dialogs(&mut self, ctx: &egui::Context) {
        // Mode Selection Dialog
        if self.show_mode_dialog {
            self.render_mode_dialog(ctx);
        }

        // Wallet Dialog
        if self.show_wallet_dialog {
            self.render_wallet_dialog(ctx);
        }

        // Buy Dialog
        if let Some(chunk_id) = self.show_buy_dialog {
            self.render_buy_dialog(ctx, chunk_id);
        }

        // Sell Dialog
        if let Some(chunk_id) = self.show_sell_dialog {
            self.render_sell_dialog(ctx, chunk_id);
        }

        // Stake Dialog
        if let Some(chunk_id) = self.show_stake_dialog {
            self.render_stake_dialog(ctx, chunk_id);
        }

        // Submit Dialog with content filter check
        if self.show_submit_dialog {
            self.render_submit_dialog(ctx);
        }

        // Transaction Status Toast
        self.render_transaction_status(ctx);
        
        // Transaction Error Display
        if let Some(ref error) = self.last_transaction_error {
            self.render_transaction_error(ctx, error);
        }
    }

    /// Wallet Connect Dialog
    fn render_wallet_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Connect Wallet")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 300.0))
            .show(ctx, |ui| {
                ui.label("Enter your Alephium wallet address:");
                
                let mut address = self.wallet.address.clone().unwrap_or_default();
                ui.text_edit_singleline(&mut address);
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() && !address.is_empty() {
                        self.wallet.connect(address);
                        self.show_wallet_dialog = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_wallet_dialog = false;
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.label("⚠️ Rules:");
                ui.label("- Only connect wallets you own");
                ui.label("- Never share your private keys");
                ui.label("- This is a demo - real wallet integration requires Alephium Web3");
            });
    }

    /// Buy Chunk Dialog
    fn render_buy_dialog(&mut self, ctx: &egui::Context, chunk_id: u32) {
        egui::Window::new("Buy Chunk")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 350.0))
            .show(ctx, |ui| {
                ui.heading(format!("Buy Chunk #{}", chunk_id));
                
                // Get chunk info
                let price = 1_000_000_000_000_000_000u64; // 1 ALPH (mock)
                let treasury_fee = (price as f64 * 0.95) as u64;
                let referrer_fee = (price as f64 * 0.05) as u64;
                
                ui.label(format!("Price: {} ALPH", price as f64 / 1e18));
                ui.label(format!("Treasury Fee (95%): {} ALPH", treasury_fee as f64 / 1e18));
                ui.label(format!("Referrer Fee (5%): {} ALPH", referrer_fee as f64 / 1e18));
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.label("📋 Rules:");
                ui.label("- Initial purchase: Treasury gets 95%, Referrer 5%");
                ui.label("- Price increases by 1 ALPH after each random purchase");
                ui.label("- You will own this chunk exclusively");
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Confirm Purchase").clicked() {
                        // Create buy transaction
                        if let Some(contract) = self.wallet.get_contract() {
                            // In real implementation, call contract.buy_chunk()
                            println!("Buying chunk {}", chunk_id);
                        }
                        self.show_buy_dialog = None;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_buy_dialog = None;
                    }
                });
            });
    }

    /// Sell Chunk Dialog
    fn render_sell_dialog(&mut self, ctx: &egui::Context, chunk_id: u32) {
        static mut SELL_PRICE: u64 = 1_000_000_000_000_000_000;
        
        egui::Window::new("Sell Chunk")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 350.0))
            .show(ctx, |ui| {
                ui.heading(format!("Sell Chunk #{}", chunk_id));
                
                ui.label("Set your price (in ALPH):");
                unsafe {
                    let mut price_alph = SELL_PRICE as f64 / 1e18;
                    ui.add(egui::DragValue::new(&mut price_alph).speed(0.1).prefix("ALPH: "));
                    SELL_PRICE = (price_alph * 1e18) as u64;
                }
                
                let price = unsafe { SELL_PRICE };
                let seller_reward = (price as f64 * 0.95) as u64;
                let treasury_fee = (price as f64 * 0.04) as u64;
                let referrer_fee = (price as f64 * 0.01) as u64;
                
                ui.add_space(10.0);
                ui.label("Fee breakdown:");
                ui.label(format!("You receive (95%): {} ALPH", seller_reward as f64 / 1e18));
                ui.label(format!("Treasury (4%): {} ALPH", treasury_fee as f64 / 1e18));
                ui.label(format!("Referrer (1%): {} ALPH", referrer_fee as f64 / 1e18));
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.label("📋 Rules:");
                ui.label("- Secondary sale: Seller 95%, Treasury 4%, Referrer 1%");
                ui.label("- Chunk ownership transfers immediately upon sale");
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("List for Sale").clicked() {
                        // Create sell transaction
                        self.show_sell_dialog = None;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_sell_dialog = None;
                    }
                });
            });
    }

    /// Stake/Bid Dialog for Auction Chunks
    fn render_stake_dialog(&mut self, ctx: &egui::Context, chunk_id: u32) {
        static mut BID_AMOUNT: u64 = 1_000_000_000_000_000_000;
        
        egui::Window::new("Place Bid")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(400.0, 400.0))
            .show(ctx, |ui| {
                ui.heading(format!("Bid on Auction Chunk #{}", chunk_id));
                
                ui.label("Your bid amount (in ALPH):");
                unsafe {
                    let mut amount_alph = BID_AMOUNT as f64 / 1e18;
                    ui.add(egui::DragValue::new(&mut amount_alph).speed(0.1).prefix("ALPH: "));
                    BID_AMOUNT = (amount_alph * 1e18) as u64;
                }
                
                ui.add_space(10.0);
                ui.label("⚠️ Minimum bid: 1 ALPH");
                ui.label("⚠️ Maximum bid: Unlimited");
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.label("📋 Auction Rules:");
                ui.label("- Highest bidder wins when auction ends");
                ui.label("- Auction ends 1,000 blocks before Alepe's next jump");
                ui.label("- Losing bids go to Treasury");
                ui.label("- Winner pays according to fee table and owns the chunk");
                ui.label("- Equal bids: winner chosen randomly");
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Place Bid").clicked() {
                        // Create bid transaction
                        self.show_stake_dialog = None;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_stake_dialog = None;
                    }
                });
            });
    }

    /// Submit Pixel Changes Dialog
    fn render_submit_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Submit Changes")
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(450.0, 400.0))
            .show(ctx, |ui| {
                ui.heading("Submit Pixel Changes");
                
                ui.label(format!("Pending changes: {} pixels", self.pending_changes.len()));
                
                // Calculate gas estimate
                let gas_estimate = self.pending_changes.len() as u64 * 1000;
                ui.label(format!("Estimated gas: {} ALPH", gas_estimate as f64 / 1e18));
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.label("📋 Rules & Agreement:");
                ui.label("- By submitting, you agree that changes are permanent");
                ui.label("- Only submit appropriate content");
                ui.label("- You must own the chunks you're modifying");
                ui.label("- Transaction fee applies");
                ui.label("- Changes are recorded in blockchain history");
                
                ui.add_space(10.0);
                ui.label("⚠️ Content Warning:");
                ui.label("Client providers may filter inappropriate content.");
                ui.label("Inappropriate content may be removed in certain jurisdictions.");
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Confirm & Submit").clicked() {
                        // Create submit pixels transaction
                        self.submit_pixels_to_blockchain();
                        self.show_submit_dialog = false;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_submit_dialog = false;
                    }
                });
            });
    }

    /// Transaction Status Toast
    fn render_transaction_status(&mut self, ctx: &egui::Context) {
        match &self.transaction_status {
            TransactionStatus::Pending => {}
            TransactionStatus::Submitted(tx_id) => {
                egui::Area::new("tx_status")
                    .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
                    .show(ctx, |ui| {
                        egui::Frame::panel()
                            .fill(egui::Color32::from_rgb(50, 50, 200))
                            .rounding(10.0)
                            .show(ui, |ui| {
                                ui.label(format!("⏳ Transaction submitted: {}", tx_id));
                            });
                    });
            }
            TransactionStatus::Confirmed(block_num) => {
                egui::Area::new("tx_status")
                    .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
                    .show(ctx, |ui| {
                        egui::Frame::panel()
                            .fill(egui::Color32::from_rgb(50, 200, 50))
                            .rounding(10.0)
                            .show(ui, |ui| {
                                ui.label(format!("✅ Confirmed in block {}", block_num));
                            });
                    });
            }
            TransactionStatus::Failed(error) => {
                egui::Area::new("tx_status")
                    .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
                    .show(ctx, |ui| {
                        egui::Frame::panel()
                            .fill(egui::Color32::from_rgb(200, 50, 50))
                            .rounding(10.0)
                            .show(ui, |ui| {
                                ui.label(format!("❌ Failed: {}", error));
                            });
                    });
            }
        }
    }

    /// Submit pending pixel changes to blockchain
    fn submit_pixels_to_blockchain(&mut self) {
        if self.pending_changes.is_empty() {
            return;
        }

        // Group changes by chunk
        let mut changes_by_chunk: HashMap<u32, Vec<&PixelChange>> = HashMap::new();
        for change in &self.pending_changes {
            changes_by_chunk.entry(change.chunk_id).or_insert_with(Vec::new).push(change);
        }

        // Create transactions for each chunk
        for (chunk_id, changes) in changes_by_chunk {
            if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                // Apply changes to chunk
                for change in changes {
                    chunk.set_pixel(change.x, change.y, change.color);
                }
                
                // In real implementation: create transaction via contract
                // let tx_builder = TransactionBuilder::new(self.wallet.address.clone().unwrap());
                // let tx_data = tx_builder.build_submit_pixels(chunk_id, chunk.pixels.to_vec());
                
                self.transaction_status = TransactionStatus::Submitted("mock_tx".to_string());
            }
        }

        // Clear pending changes
        self.pending_changes.clear();
    }
}
