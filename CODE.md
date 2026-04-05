# Alepic Programmer Manual

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Module Reference](#module-reference)
4. [Data Structures](#data-structures)
5. [Extending Alepic](#extending-alepic)
6. [Smart Contract Integration](#smart-contract-integration)
7. [Best Practices](#best-practices)
8. [API Reference](#api-reference)

---

## Overview

Alepic is a decentralized collaborative canvas application built on the Alephium blockchain. Users can buy, sell, and draw on 16×16 pixel chunks in a 4096×2160 (UHD/4K) canvas using a 4-bit color palette (16 colors).

### Key Features

- **Collaborative Canvas**: 34,560 chunks (256×135 grid) of 16×16 pixels each
- **4-bit Color System**: 16-color indexed palette
- **Blockchain Integration**: Full Alephium smart contract integration
- **The Alepe Game**: Mascot jumps every 100,000 blocks, rewarding chunk owners
- **Market System**: Buy, sell, and auction mechanics with fee distribution
- **Billboard Mode**: Read-only public display mode
- **Content Filtering**: Moderation system with neural network support

### Technology Stack

- **Language**: Rust 2021 Edition
- **GUI Framework**: Eframe/Egui (immediate mode GUI)
- **Blockchain**: Alephium (Ralph smart contracts)
- **HTTP Client**: Reqwest for REST API calls
- **Async Runtime**: Tokio
- **Serialization**: Serde/serde_json

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Alepic Desktop Client                     │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   UI Layer   │  │  Rendering   │  │    Game      │      │
│  │  (Egui)      │  │  (Textures)  │  │   (Alepe)    │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                 │               │
│  ┌──────┴─────────────────┴─────────────────┴───────┐      │
│  │              Application Core (app.rs)            │      │
│  └──────┬─────────────────┬─────────────────┬───────┘      │
│         │                 │                 │               │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────┴───────┐     │
│  │   Canvas     │  │  Blockchain  │  │   Content    │     │
│  │   Manager    │  │   Manager    │  │   Filter     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Alephium Blockchain                         │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐         ┌──────────────────┐         │
│  │  Smart Contract  │◄───────►│   User Wallet    │         │
│  │  (Ralph)         │         │                  │         │
│  └────────┬─────────┘         └──────────────────┘         │
│           │                                                 │
│  ┌────────┴─────────┐                                      │
│  │     Treasury     │                                      │
│  └──────────────────┘                                      │
└─────────────────────────────────────────────────────────────┘
```

### File Structure

```
/workspace
├── Cargo.toml                      # Project dependencies
├── README.md                       # Design document
├── BUILD.md                        # Build instructions
├── CODE.md                         # This file
├── build.sh                        # Build script
├── contracts/
│   └── AlepicMain.ralph           # Smart contract source
├── assets/
│   ├── images/                     # Image assets
│   └── sounds/                     # Sound effects
└── src/
    ├── main.rs                     # Entry point
    ├── app.rs                      # Application core
    ├── mode.rs                     # Operation modes
    ├── utils.rs                    # Utilities
    ├── canvas/
    │   ├── mod.rs                  # Canvas module
    │   └── chunk.rs                # Chunk data structure
    ├── ui/
    │   ├── mod.rs                  # UI module
    │   ├── color_layer.rs          # Color layer UI
    │   └── market_layer.rs         # Market layer UI
    ├── rendering/
    │   ├── mod.rs                  # Rendering module
    │   └── texture_mgr.rs          # Texture management
    ├── blockchain/
    │   ├── mod.rs                  # Blockchain module
    │   ├── client.rs               # Alephium HTTP client
    │   ├── contract.rs             # Contract interface
    │   ├── manager.rs              # Blockchain manager
    │   ├── wallet.rs               # Wallet management
    │   ├── fees.rs                 # Fee calculations
    │   └── transactions.rs         # Transaction handling
    ├── game/
    │   ├── mod.rs                  # Game module
    │   └── alepe.rs                # Alepe mascot logic
    ├── content_filter.rs           # Content moderation
    └── billboard.rs                # Billboard display mode
```

### Data Flow

1. **User Input** → `app.rs::handle_input()` processes zoom/pan/draw
2. **Canvas Rendering** → `rendering/texture_mgr.rs` updates textures
3. **UI Rendering** → `ui/color_layer.rs` or `ui/market_layer.rs`
4. **Transaction Request** → `blockchain/contract.rs` builds transaction
5. **Blockchain Submission** → `blockchain/client.rs` sends to Alephium node
6. **State Update** → Contract state changes, events emitted
7. **Reconstruction** → Client fetches updated state from blockchain

---

## Module Reference

### Core Modules

#### `main.rs` - Entry Point

The application entry point initializes the Eframe window and creates the main application instance.

```rust
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Alepic - Alephium Collaborative Canvas",
        native_options,
        Box::new(|cc| Ok(Box::new(AlepicApp::new(cc)))),
    )
}
```

**Key Responsibilities:**
- Window configuration
- Application initialization
- Event loop startup

#### `app.rs` - Application Core

The central hub managing all application state and coordinating between modules.

**Main Struct: `AlepicApp`**

```rust
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
    // ... dialog states and transaction tracking
}
```

**Key Methods:**

| Method | Purpose |
|--------|---------|
| `new(cc)` | Initialize application with default state |
| `init_blockchain(url, address)` | Configure blockchain connection |
| `toggle_mode()` | Switch between Real/Simulation mode |
| `enable_billboard_mode()` | Enable read-only display |
| `handle_input(ctx)` | Process user input (zoom/pan) |
| `render_canvas(ui)` | Draw the canvas with visible chunks |
| `render_layer_ui(ctx)` | Render layer-specific UI elements |
| `render_dialogs(ctx)` | Show transaction dialogs |

**Update Loop:**

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // 1. Handle input (Zoom/Pan)
    self.handle_input(ctx);

    // 2. Render Canvas (Central Panel)
    egui::CentralPanel::default().show(ctx, |ui| {
        self.render_canvas(ui);
    });

    // 3. Render Layer UI
    self.render_layer_ui(ctx);

    // 4. Check Alepe events
    self.alepe.check_jump(self.current_block);

    // 5. Render Dialogs
    self.render_dialogs(ctx);
}
```

#### `mode.rs` - Operation Modes

Defines the operational mode of the application.

```rust
pub enum OperationMode {
    /// Connect to real Alephium network
    Real,
    /// Mock blockchain interactions for testing
    Simulation,
}
```

---

### Canvas Module (`canvas/`)

#### `chunk.rs` - Chunk Data Structure

Represents a 16×16 pixel chunk with ownership and color data.

**Constants:**
```rust
pub const CHUNK_SIZE: u16 = 16;
pub const PIXELS_PER_CHUNK: usize = 256; // 16×16
```

**Struct `Chunk`:**
```rust
pub struct Chunk {
    pub id: u32,                    // Unique chunk identifier
    pub grid_x: u16,                // X position in grid (0-255)
    pub grid_y: u16,                // Y position in grid (0-134)
    pub owner: Option<String>,      // Alephium address
    pub pixels: [ColorIndex; 256],  // 4-bit color indices
    pub version: u64,               // For conflict detection
    pub is_dirty: bool,             // Local modifications pending
}
```

**Key Methods:**

| Method | Description |
|--------|-------------|
| `new(id, x, y)` | Create new empty chunk |
| `set_pixel(x, y, color)` | Set pixel color (marks dirty) |
| `to_texture_data()` | Convert to RGBA for GPU |

**Palette System:**

```rust
pub struct Palette;

impl Palette {
    pub fn get_color(index: u8) -> egui::Color32 {
        const PALETTE: [(u8, u8, u8); 16] = [
            (0, 0, 0), (255, 255, 255), (255, 0, 0), (0, 255, 0),
            (0, 0, 255), (255, 255, 0), (0, 255, 255), (255, 0, 255),
            (128, 128, 128), (192, 192, 192), (128, 0, 0), (0, 128, 0),
            (0, 0, 128), (128, 128, 0), (0, 128, 128), (128, 0, 128),
        ];
        // Returns egui::Color32 from index
    }
}
```

**Palette Layout:**
- Row 1 (0-7): Bright colors
- Row 2 (8-15): Dark colors

---

### UI Module (`ui/`)

#### `color_layer.rs` - Drawing Interface

Provides the color palette and submit button for the drawing layer.

**Function: `render_color_layer()`**

```rust
pub fn render_color_layer(
    ctx: &egui::Context, 
    selected_color: &mut u8, 
    on_submit: &mut dyn FnMut()
)
```

**UI Elements:**
1. **Color Palette** (Bottom Center)
   - Two rows of 8 colors each
   - Click to select color
   - Highlight shows current selection

2. **Submit Button** (Bottom Right)
   - Triggers submission dialog
   - Commits local changes to blockchain

#### `market_layer.rs` - Market Interface

Provides buying, selling, and auction UI elements.

**Struct: `MarketLayerConfig`**
```rust
pub struct MarketLayerConfig {
    pub show_prices: bool,
    pub show_auction_highlights: bool,
    pub show_ownership_info: bool,
}
```

**Key Functions:**

| Function | Purpose |
|----------|---------|
| `render_market_layer()` | Main market UI (wallet button) |
| `render_chunk_price_label()` | Display price above chunk |
| `render_auction_highlight()` | Orange border for auction chunks |
| `render_ownership_indicator()` | Corner marker for owned chunks |

---

### Rendering Module (`rendering/`)

#### `texture_mgr.rs` - Texture Management

Handles GPU texture creation and updates for chunks.

**Struct: `TextureManager`**

```rust
pub struct TextureManager {
    textures: HashMap<u32, egui::TextureHandle>,
    egui_ctx: egui::Context,
}
```

**Key Methods:**

| Method | Purpose |
|--------|---------|
| `new(ctx)` | Initialize with egui context |
| `update_chunk(chunk)` | Update texture if chunk changed |
| `get_texture(chunk_id)` | Get texture handle for rendering |
| `cleanup(visible_chunks)` | Remove off-screen textures |

**Optimization Strategy:**
- Textures only update when `chunk.is_dirty = true`
- Off-screen textures are cleaned up to save memory
- Batching reduces GPU calls

---

### Blockchain Module (`blockchain/`)

#### `client.rs` - Alephium HTTP Client

Low-level HTTP client for Alephium node communication.

**Struct: `AlephiumClient`**

```rust
pub struct AlephiumClient {
    node_url: String,
    http_client: reqwest::Client,
}
```

**Key Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_current_block()` | `BlockInfo` | Current block number |
| `get_chunk_owner(chunk_id)` | `Option<String>` | Chunk owner address |
| `get_treasury_balance()` | `u64` | Treasury balance in ALPH |
| `get_alepe_state()` | `AlepeState` | Alepe position/state |
| `submit_transaction()` | `String` | Submit tx, return tx ID |

#### `contract.rs` - Smart Contract Interface

High-level interface to the Alepic smart contract.

**Struct: `AlepicContract`**

```rust
pub struct AlepicContract {
    contract_address: String,
    client: AlephiumClient,
}
```

**Contract Operations:**

| Method | Parameters | Description |
|--------|------------|-------------|
| `get_chunk_info(chunk_id)` | `u32` | Get chunk ownership/price |
| `buy_random_chunk(address)` | `&str` | Buy random unowned chunk |
| `buy_chunk(address, id, price)` | `&str, u32, u64` | Buy specific chunk |
| `sell_chunk(address, id, price)` | `&str, u32, u64` | List chunk for sale |
| `place_bid(address, id, amount)` | `&str, u32, u64` | Bid on auction chunk |
| `submit_pixels(address, id, pixels)` | `&str, u32, Vec<u8>` | Update chunk pixels |
| `claim_alepe_reward(address, id)` | `&str, u32` | Claim Alepe landing reward |
| `get_auction_info(chunk_id)` | `u32` | Get auction state |
| `get_treasury_info()` | - | Get treasury statistics |

**Fee Calculations:**

From `fees.rs`:

```rust
// Initial Sale (Unowned Chunk)
// Treasury: 95%, Referrer: 5%, Seller: 0%
pub fn calculate_initial_sale(price: u64, referrer: bool) -> (u64, u64, u64)

// Secondary Sale (Owned Chunk)
// Seller: 95%, Treasury: 4%, Referrer: 1%
pub fn calculate_secondary_sale(price: u64, referrer: bool) -> (u64, u64, u64)
```

#### `manager.rs` - Blockchain Manager

Coordinates blockchain operations with mode handling.

**Struct: `BlockchainManager`**

```rust
pub struct BlockchainManager {
    mode: OperationMode,
    contract: Option<AlepicContract>,
    // Simulation state for offline testing
    simulated_chunks: HashMap<u32, ChunkInfo>,
}
```

**Features:**
- Transparent switching between Real/Simulation mode
- Mock responses in simulation mode
- Transaction queuing and status tracking

#### `wallet.rs` - Wallet Management

Manages wallet connection and signing.

**Struct: `WalletManager`**

```rust
pub struct WalletManager {
    address: Option<String>,
    connected: bool,
    // In production: secure key storage via keyring crate
}
```

**Methods:**
- `connect(address)` - Connect wallet
- `disconnect()` - Disconnect wallet
- `is_connected()` - Check connection status
- `get_contract()` - Get contract instance

#### `transactions.rs` - Transaction Handling

Transaction building and status tracking.

**Enum: `TransactionStatus`**
```rust
pub enum TransactionStatus {
    Pending,
    Submitted(String), // TxHash
    Confirmed,
    Failed(String),    // Error message
}
```

---

### Game Module (`game/`)

#### `alepe.rs` - Alepe Mascot Logic

Implements the Alepe jump mechanics and reward system.

**Constants:**
```rust
const MIN_DISTANCE: u16 = 48;   // pixels
const MAX_DISTANCE: u16 = 96;   // pixels
const JUMP_INTERVAL_BLOCKS: u64 = 100_000;
```

**Struct: `Alepe`**

```rust
pub struct Alepe {
    pub grid_x: u16,              // Position in chunks (0-255)
    pub grid_y: u16,              // Position in chunks (0-134)
    pub last_jump_block: u64,     // Block of last jump
    pub pixel_offset_x: u16,      // Offset within chunk
    pub pixel_offset_y: u16,      // Offset within chunk
}
```

**Jump Algorithm (from README.md):**

```rust
fn jump(&mut self, current_block: u64) {
    let mut rng = rand::thread_rng();
    
    // Distance in pixels (48-96)
    let dx_pixels = rng.gen_range(MIN_DISTANCE..=MAX_DISTANCE);
    let dy_pixels = rng.gen_range(MIN_DISTANCE..=MAX_DISTANCE);
    
    // Random direction
    let sign_x: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
    let sign_y: i8 = if rng.gen_bool(0.5) { 1 } else { -1 };
    
    // Convert to chunks and apply with wrapping
    let dx_chunks = dx_pixels / CHUNK_SIZE_PIXELS;
    let dy_chunks = dy_pixels / CHUNK_SIZE_PIXELS;
    
    // Apply movement with toroidal wrapping
    if sign_x > 0 {
        self.grid_x = (self.grid_x as u32 + dx_chunks as u32) % CANVAS_WIDTH_CHUNKS as u32 as u16;
    } else {
        self.grid_x = (self.grid_x as i32 - dx_chunks as i32)
            .rem_euclid(CANVAS_WIDTH_CHUNKS as i32) as u16;
    }
    // Same for Y axis...
    
    self.last_jump_block = current_block;
}
```

**Key Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `check_jump(current_block)` | `bool` | Check if jump needed, execute |
| `get_pixel_position()` | `(u32, u32)` | Exact pixel coordinates |
| `occupies_chunk(x, y)` | `bool` | Check if Alepe on chunk (2×2 area) |
| `get_auction_chunks()` | `Vec<(u16,u16)>` | Get chunks for auction |
| `blocks_until_jump(current_block)` | `u64` | Blocks remaining until jump |

**Reward System:**
- When Alepe lands on a chunk, owner receives 1% of treasury
- Reward must be claimed manually via `claim_alepe_reward()`
- Auction starts for surrounding chunks 1000 blocks before next jump

---

### Content Filter Module

#### `content_filter.rs` - Moderation System

Provides content filtering for inappropriate material.

**Struct: `ContentFilter`**

```rust
pub struct ContentFilter {
    enabled: bool,
    strict_mode: bool,
    blocked_patterns: HashSet<Vec<u8>>,
}
```

**Detection Heuristics:**

1. **Solid Color Detection**: Flags chunks with >250 pixels of same color (spam)
2. **Flashing Pattern Detection**: Identifies rapid alternation between 2 colors
3. **Text-like Pattern Detection**: Finds long horizontal/vertical lines

**Advanced Filter with Neural Network:**

```rust
pub struct AdvancedContentFilter {
    base_filter: ContentFilter,
    use_nn: bool,
    nn_service_url: Option<String>,
}
```

**Usage:**
```rust
let mut filter = AdvancedContentFilter::new();
filter.enable_neural_network("http://nn-service:8080/check".to_string());

let result = filter.check_content(&pixels, chunk_id).await;
match result {
    ModerationResult::Approved => { /* OK */ }
    ModerationResult::Rejected(reason) => { /* Block */ }
    ModerationResult::NeedsReview(reason) => { /* Queue for manual review */ }
}
```

---

### Billboard Module

#### `billboard.rs` - Public Display Mode

Read-only display mode for public installations.

**Struct: `BillboardMode`**

```rust
pub struct BillboardMode {
    enabled: bool,
    refresh_interval: u64,    // Seconds between refreshes
    show_alepe: bool,         // Show Alepe indicator
    show_info_overlay: bool,  // Show info panel
}
```

**Features:**
- No UI elements (no buttons, palettes, dialogs)
- Zoom and pan only
- Auto-refresh at configurable intervals
- Optional info overlay with timestamp

**Display Types:**

```rust
pub enum DisplayType {
    Monitor,      // 16:9
    Vertical,     // 9:16
    Ultrawide,    // 21:9
    Custom(f32, f32),
}
```

**Public Display Manager:**

```rust
pub struct PublicDisplayManager {
    billboard_mode: BillboardMode,
    display_type: DisplayType,
    screen_width: f32,
    screen_height: f32,
    auto_cycle: bool,        // Auto-cycle views
    cycle_interval: u64,     // Seconds per view
}
```

---

## Data Structures

### Coordinate Systems

**Canvas Coordinates:**
- Total size: 4096 × 2160 pixels
- Chunk grid: 256 × 135 chunks
- Each chunk: 16 × 16 pixels

**Coordinate Conversion:**
```rust
// Chunk ID to Grid Position
grid_x = chunk_id % 256
grid_y = chunk_id / 256

// Grid Position to Pixel Position
pixel_x = grid_x * 16 + offset_x
pixel_y = grid_y * 16 + offset_y

// Pixel Position to Chunk ID
chunk_id = (pixel_x / 16) + (pixel_y / 16) * 256
```

### Chunk Info Structure

```rust
pub struct ChunkInfo {
    pub chunk_id: u32,
    pub grid_x: u16,
    pub grid_y: u16,
    pub owner: Option<String>,
    pub price: u64,              // In ALPH (smallest unit: 10^-18)
    pub is_auction_chunk: bool,
    pub last_update_block: u64,
}
```

### Auction Info Structure

```rust
pub struct AuctionInfo {
    pub chunk_id: u32,
    pub highest_bid: u64,
    pub highest_bidder: Option<String>,
    pub auction_end_block: u64,
    pub bids_count: u32,
}
```

### Treasury Info Structure

```rust
pub struct TreasuryInfo {
    pub total_balance: u64,
    pub alepe_rewards_paid: u64,
    pub total_chunks_sold: u32,
}
```

---

## Extending Alepic

### Adding New UI Layers

To add a new view layer:

1. **Add Layer Type Enum** in `app.rs`:
```rust
enum LayerType {
    Color,
    Market,
    Analytics,  // New layer
}
```

2. **Create UI Module** in `ui/analytics_layer.rs`:
```rust
use eframe::egui;

pub fn render_analytics_layer(ctx: &egui::Context, on_back: &mut dyn FnMut()) {
    // Back button
    egui::Area::new("layer_switch")
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(20.0, -20.0))
        .show(ctx, |ui| {
            if ui.button("📊 Back to Canvas").clicked() {
                on_back();
            }
        });
    
    // Your analytics UI here
}
```

3. **Update Module Declaration** in `ui/mod.rs`:
```rust
pub mod color_layer;
pub mod market_layer;
pub mod analytics_layer;  // Add this
```

4. **Implement Render Logic** in `app.rs`:
```rust
fn render_layer_ui(&mut self, ctx: &egui::Context) {
    match self.current_layer {
        LayerType::Color => self.render_color_layer(ctx),
        LayerType::Market => self.render_market_layer(ctx),
        LayerType::Analytics => self.render_analytics_layer(ctx),
    }
}
```

### Adding New Blockchain Operations

To add a new smart contract function:

1. **Add Method to `AlepicContract`** in `blockchain/contract.rs`:
```rust
pub async fn new_operation(
    &self,
    user_address: &str,
    param1: u32,
    param2: u64,
) -> Result<TransactionResult, Box<dyn std::error::Error>> {
    // Validate parameters
    
    // Encode function call
    let call_data = self.encode_new_operation_call(param1, param2)?;
    
    // Submit transaction
    let tx_id = self.client
        .submit_transaction(user_address, &self.contract_address, 0, Some(call_data))
        .await?;
    
    Ok(TransactionResult {
        success: true,
        transaction_id: tx_id,
        message: "Operation completed".to_string(),
    })
}
```

2. **Add Encoding Helper**:
```rust
fn encode_new_operation_call(&self, param1: u32, param2: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    data.extend_from_slice(&[0x67, 0x89, 0xAB, 0xCD]); // Function selector
    data.extend_from_slice(&param1.to_be_bytes());
    let mut price_bytes = [0u8; 32];
    price_bytes[24..].copy_from_slice(&param2.to_be_bytes());
    data.extend_from_slice(&price_bytes);
    Ok(data)
}
```

3. **Update Smart Contract** in `contracts/AlepicMain.ralph`:
```ralph
@usingInterface!(IErc1155Like)
contract AlepicMain extends IErc1155Like {
    // ... existing code ...
    
    pub fn newOperation(param1: U256, param2: U256) -> () {
        // Implementation
    }
}
```

### Integrating Neural Network Content Filter

To integrate an external NN service:

1. **Implement NN Service Client**:
```rust
pub struct NeuralNetworkService {
    client: reqwest::Client,
    endpoint: String,
}

impl NeuralNetworkService {
    pub async fn check_content(&self, pixels: &[u8]) -> Result<bool, String> {
        let response = self.client
            .post(&self.endpoint)
            .json(&serde_json::json!({ "pixels": pixels }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(result["appropriate"].as_bool().unwrap_or(true))
    }
}
```

2. **Integrate with AdvancedContentFilter**:
```rust
impl AdvancedContentFilter {
    pub fn set_neural_network_service(&mut self, service: NeuralNetworkService) {
        self.nn_service = Some(service);
        self.use_nn = true;
    }
    
    async fn check_with_neural_network(&self, pixels: &[u8]) -> Result<bool, String> {
        if let Some(ref service) = self.nn_service {
            service.check_content(pixels).await
        } else {
            Err("No NN service configured".to_string())
        }
    }
}
```

### Adding Sound Effects

1. **Add Audio Crate** to `Cargo.toml`:
```toml
rodio = "0.17"  # Audio playback
```

2. **Create Sound Manager** in `src/audio.rs`:
```rust
use rodio::{Sink, Source};
use std::collections::HashMap;
use std::io::BufReader;

pub struct SoundManager {
    sounds: HashMap<String, Vec<u8>>,
    sink: Option<Sink>,
}

impl SoundManager {
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
            sink: None,
        }
    }
    
    pub fn load_sound(&mut self, name: &str, data: Vec<u8>) {
        self.sounds.insert(name.to_string(), data);
    }
    
    pub fn play(&mut self, name: &str) {
        if let Some(data) = self.sounds.get(name) {
            // Play sound using rodio
        }
    }
}
```

3. **Trigger Sounds** in appropriate places:
```rust
// In alepe.rs after jump
println!("Alepe jumped!");
sound_manager.play("jump");
```

---

## Smart Contract Integration

### Ralph Contract Structure

The smart contract (`contracts/AlepicMain.ralph`) implements:

1. **Chunk Ownership Tracking**
2. **Price Mechanism** (increasing price for random purchases)
3. **Fee Distribution** (Treasury, Seller, Referrer)
4. **Auction System**
5. **Alepe Game Logic**
6. **Pixel Data Storage**

### Contract State Variables

```ralph
contract AlepicMain {
    // Chunk ownership: chunkId -> owner
    Map<U256, ByteVec> chunkOwners
    
    // Chunk prices: chunkId -> price
    Map<U256, U256> chunkPrices
    
    // Next random chunk price
    U256 nextRandomPrice
    
    // Alepe state
    U256 alepeX
    U256 alepeY
    U256 lastJumpBlock
    
    // Treasury
    U256 treasuryBalance
    
    // Auction state
    Map<U256, Auction> auctions
}
```

### Calling Contract from Rust

Example: Buying a chunk

```rust
// 1. Get contract instance
let contract = wallet.get_contract()?;

// 2. Call contract method
let result = contract.buy_chunk(
    wallet.address.as_ref().unwrap(),
    chunk_id,
    price,
).await?;

// 3. Handle result
if result.success {
    println!("Transaction submitted: {}", result.transaction_id);
} else {
    eprintln!("Transaction failed: {}", result.message);
}
```

### Transaction Lifecycle

1. **Build**: Create transaction data with function selector and parameters
2. **Sign**: User signs with private key (via wallet)
3. **Submit**: Send to Alephium node via HTTP API
4. **Pending**: Wait for inclusion in block
5. **Confirm**: Verify execution success
6. **Update**: Fetch updated state from blockchain

### Event Handling

The contract emits events for:
- `ChunkPurchased(chunkId, buyer, price)`
- `ChunkSold(chunkId, seller, buyer, price)`
- `PixelsUpdated(chunkId, owner)`
- `AlepeJumped(newX, newY, blockNumber)`
- `AuctionBid(chunkId, bidder, amount)`
- `RewardClaimed(chunkId, owner, amount)`

Subscribe to events:

```rust
let events = client.subscribe_events(contract_address).await?;
while let Some(event) = events.next().await {
    match event.name.as_str() {
        "ChunkPurchased" => { /* Handle purchase */ }
        "AlepeJumped" => { /* Update Alepe position */ }
        _ => {}
    }
}
```

---

## Best Practices

### Performance Optimization

1. **Texture Management**
   - Only update textures when chunk changes
   - Clean up off-screen textures
   - Batch texture updates

2. **Chunk Loading**
   - Load only visible chunks
   - Use spatial hashing for quick lookup
   - Implement Level of Detail (LOD) for zoomed-out views

3. **Blockchain Calls**
   - Cache chunk info locally
   - Batch multiple queries
   - Use websockets for real-time updates

### Security Considerations

1. **Wallet Security**
   - Never store private keys in plaintext
   - Use `keyring` crate for secure storage
   - Validate all wallet addresses

2. **Transaction Validation**
   - Verify chunk ownership before operations
   - Check balances before submitting
   - Implement retry logic for failed transactions

3. **Content Moderation**
   - Enable content filter by default
   - Provide reporting mechanism
   - Support jurisdiction-specific rules

### Code Organization

1. **Separation of Concerns**
   - UI logic in `ui/` modules
   - Business logic in `app.rs`
   - Blockchain interaction in `blockchain/` modules

2. **Error Handling**
   - Use `Result<T, E>` for fallible operations
   - Provide meaningful error messages
   - Log errors for debugging

3. **Testing**
   - Unit tests for pure functions
   - Integration tests for blockchain operations
   - UI tests with egui testing utilities

### Documentation

1. **Code Comments**
   - Document public APIs
   - Explain complex algorithms
   - Include examples where helpful

2. **Inline Documentation**
   - Use Rust doc comments (`///`)
   - Generate docs with `cargo doc`
   - Keep docs up to date

---

## API Reference

### Quick Reference Table

| Module | Key Structs | Key Functions |
|--------|-------------|---------------|
| `app` | `AlepicApp`, `Viewport` | `new()`, `update()`, `render_canvas()` |
| `canvas::chunk` | `Chunk`, `Palette` | `new()`, `set_pixel()`, `get_color()` |
| `ui::color_layer` | - | `render_color_layer()` |
| `ui::market_layer` | `MarketLayerConfig` | `render_market_layer()`, `render_chunk_price_label()` |
| `rendering::texture_mgr` | `TextureManager` | `new()`, `update_chunk()`, `get_texture()` |
| `blockchain::client` | `AlephiumClient` | `get_current_block()`, `submit_transaction()` |
| `blockchain::contract` | `AlepicContract`, `ChunkInfo` | `buy_chunk()`, `sell_chunk()`, `get_chunk_info()` |
| `blockchain::fees` | `FeeCalculator` | `calculate_initial_sale()`, `calculate_secondary_sale()` |
| `game::alepe` | `Alepe` | `new()`, `check_jump()`, `get_auction_chunks()` |
| `content_filter` | `ContentFilter`, `AdvancedContentFilter` | `is_appropriate()`, `check_content()` |
| `billboard` | `BillboardMode`, `PublicDisplayManager` | `enable()`, `render_billboard()` |

### Common Patterns

**Getting Chunk from Screen Position:**
```rust
fn screen_to_chunk(screen_pos: egui::Pos2, viewport: &Viewport) -> Option<u32> {
    let world_x = (screen_pos.x - viewport.offset.x) / viewport.zoom;
    let world_y = (screen_pos.y - viewport.offset.y) / viewport.zoom;
    
    let grid_x = (world_x / 16.0) as u16;
    let grid_y = (world_y / 16.0) as u16;
    
    if grid_x < 256 && grid_y < 135 {
        Some((grid_x as u32) + (grid_y as u32) * 256)
    } else {
        None
    }
}
```

**Submitting Pixel Changes:**
```rust
async fn submit_changes(app: &mut AlepicApp) -> Result<(), String> {
    // 1. Run content filter
    for change in &app.pending_changes {
        let chunk = app.chunks.get(&change.chunk_id).ok_or("Chunk not found")?;
        let result = app.content_filter.check_content(&chunk.pixels, change.chunk_id).await;
        
        if !result.is_approved {
            return Err(format!("Content rejected: {:?}", result.reason));
        }
    }
    
    // 2. Submit to blockchain
    let contract = app.wallet.get_contract().ok_or("Wallet not connected")?;
    for change in &app.pending_changes {
        let chunk = app.chunks.get(&change.chunk_id).unwrap();
        contract.submit_pixels(
            app.wallet.address.as_ref().unwrap(),
            change.chunk_id,
            chunk.pixels.to_vec(),
        ).await?;
    }
    
    // 3. Clear pending changes
    app.pending_changes.clear();
    
    Ok(())
}
```

**Handling Alepe Jump:**
```rust
fn on_alepe_jump(app: &mut AlepicApp, new_x: u16, new_y: u16) {
    println!("Alepe jumped to ({}, {})!", new_x, new_y);
    
    // Check if user owns any of the occupied chunks
    for dx in 0..2 {
        for dy in 0..2 {
            let cx = (new_x + dx) % 256;
            let cy = (new_y + dy) % 135;
            let chunk_id = (cx as u32) + (cy as u32) * 256;
            
            if let Some(info) = app.chunk_infos.get(&chunk_id) {
                if info.owner.as_ref() == app.wallet.address.as_ref() {
                    println!("Alepe landed on your chunk! Claim reward.");
                    // Show claim reward dialog
                }
            }
        }
    }
    
    // Mark surrounding chunks as auction chunks
    let auction_chunks = app.alepe.get_auction_chunks();
    for (ax, ay) in auction_chunks {
        let chunk_id = (ax as u32) + (ay as u32) * 256;
        if let Some(info) = app.chunk_infos.get_mut(&chunk_id) {
            info.is_auction_chunk = true;
        }
    }
}
```

---

## Troubleshooting

### Common Issues

**Issue: Textures not updating**
- Check `chunk.is_dirty` flag
- Ensure `texture_mgr.update_chunk()` is called
- Verify egui context is valid

**Issue: Transactions failing**
- Check wallet connection
- Verify sufficient ALPH balance
- Review transaction error message
- Ensure contract address is correct

**Issue: Alepe not jumping**
- Verify block number is increasing
- Check `JUMP_INTERVAL_BLOCKS` constant
- Ensure `check_jump()` is called each frame

**Issue: Poor performance**
- Reduce visible chunk count
- Optimize texture cleanup
- Use release build (`--release`)

### Debug Tools

**Enable Logging:**
```rust
env_logger::init();
log::debug!("Debug message");
log::info!("Info message");
```

**Inspect State:**
```rust
// Add debug UI
egui::Window::new("Debug").show(ctx, |ui| {
    ui.label(format!("Chunks loaded: {}", app.chunks.len()));
    ui.label(format!("FPS: {}", ui.ctx().fps()));
    ui.label(format!("Alepe at: ({}, {})", app.alepe.grid_x, app.alepe.grid_y));
});
```

---

## Contributing

### Code Style

- Follow Rustfmt defaults
- Use meaningful variable names
- Document public APIs
- Write tests for new features

### Pull Request Process

1. Fork the repository
2. Create feature branch
3. Implement changes
4. Add/update tests
5. Update documentation
6. Submit PR

### Testing Requirements

- Unit tests for business logic
- Integration tests for blockchain operations
- Manual testing checklist for UI changes

### Running Tests

See [BUILD.md](BUILD.md#testing) for detailed testing instructions.

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test --test canvas_tests
cargo test --test alepe_tests
cargo test --test fees_tests
cargo test --test content_filter_tests
```

**Smart Contract Tests:**

```bash
# Install dependencies
npm install

# Run smart contract tests
npm run test:contract
```

For comprehensive testing methodology and security procedures, see [TESTING.md](TESTING.md).

---

## Test Suite Reference

The Alepic project includes a comprehensive test suite covering all major subsystems.

**Total Tests:** 95+ tests across 4 Rust test modules + smart contract tests

### Canvas Tests (`tests/canvas_tests.rs`)

**Coverage:** 20+ tests for chunk and canvas operations

| Test Category | Tests |
|--------------|-------|
| Chunk Creation | `test_chunk_creation`, `test_chunk_set_pixel`, `test_chunk_set_pixel_out_of_bounds` |
| Pixel Operations | `test_chunk_set_same_color_no_dirty`, `test_chunk_all_pixels`, `test_chunk_texture_data` |
| Canvas Management | `test_canvas_manager_creation`, `test_canvas_get_or_create_chunk` |
| Coordinate Conversion | `test_canvas_grid_to_id_conversion`, `test_canvas_id_to_grid_conversion` |
| Boundary Tests | `test_canvas_set_pixel_out_of_bounds`, `test_canvas_corner_pixels` |
| Dirty Tracking | `test_canvas_dirty_chunks` |

### Alepe Game Tests (`tests/alepe_tests.rs`)

**Coverage:** 30+ tests for game mechanics

| Test Category | Tests |
|--------------|-------|
| Initialization | `test_alepe_creation`, `test_alepe_default_trait` |
| Jump Timing | `test_alepe_no_jump_before_interval`, `test_alepe_jump_at_interval`, `test_alepe_multiple_jumps` |
| Position Calculation | `test_alepe_get_pixel_position_initial`, `test_alepe_get_pixel_position_with_offset` |
| Chunk Occupation | `test_alepe_occupies_own_chunks`, `test_alepe_does_not_occupy_distant_chunks` |
| Auction Chunks | `test_alepe_get_auction_chunks_count`, `test_alepe_auction_chunks_exclude_center` |
| Wrapping | `test_alepe_wrapping_at_right_edge`, `test_alepe_wrapping_at_left_edge` |

### Fee System Tests (`tests/fees_tests.rs`)

**Coverage:** 20+ tests for fee calculations

| Test Category | Tests |
|--------------|-------|
| Initial Sale | `test_initial_sale_with_referrer`, `test_initial_sale_without_referrer` |
| Secondary Sale | `test_secondary_sale_with_referrer`, `test_secondary_sale_without_referrer` |
| Edge Cases | `test_initial_sale_small_amounts`, `test_fee_calculation_boundary_values` |
| Accuracy | `test_fee_percentages_accuracy`, `test_consistent_behavior_multiple_calls` |

### Content Filter Tests (`tests/content_filter_tests.rs`)

**Coverage:** 25+ tests for moderation system

| Test Category | Tests |
|--------------|-------|
| Basic Filtering | `test_content_filter_creation`, `test_filter_disabled_allows_all` |
| Strict Mode | `test_strict_mode_toggle`, `test_solid_color_detection_strict` |
| Pattern Detection | `test_block_pattern`, `test_horizontal_line_detection` |
| Moderation Results | `test_moderation_result_approved`, `test_moderation_result_rejected` |
| Advanced Filter | `test_advanced_filter_creation`, `test_advanced_filter_neural_network_config` |

### Smart Contract Tests (`test/contract/`)

**Coverage:** Treasury security, marketplace logic, game mechanics

See [TESTING.md](TESTING.md) for complete smart contract test documentation including:

- **Treasury Protection Tests**: Reentrancy guards, unauthorized access prevention
- **Marketplace Tests**: Ownership transfer, auction logic, referrer handling
- **Game Logic Tests**: Jump timing, position calculation, reward distribution
- **Security Audit Checklist**: Code review, static analysis, formal verification

---

## Security Testing

**CRITICAL:** Smart contract security is paramount. The Treasury must be protected from all unauthorized access.

### Key Security Principles

1. **No Public Withdrawals**: Treasury funds can only move via protocol-defined mechanics
2. **Checks-Effects-Interactions**: State updates before external calls
3. **Invariant Preservation**: Total inputs must equal total outputs in all transactions
4. **Access Control**: Strict signer verification on all privileged functions

### Security Test Categories

| Category | Focus | Tests |
|----------|-------|-------|
| Treasury Protection | Prevent theft | Direct withdrawal attempts, fee manipulation |
| Reentrancy Guards | Prevent recursive attacks | Malicious contract callbacks |
| Fee Distribution | Verify calculations | Exact percentage accuracy, rounding |
| Access Control | Verify permissions | Unauthorized function calls |

For detailed security testing methodology, threat models, and audit procedures, see [TESTING.md](TESTING.md).

---

## License

This project is licensed under GPL-3.0. See LICENSE file for details.

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Egui Documentation](https://docs.rs/egui/)
- [Alephium Docs](https://docs.alephium.org/)
- [Ralph Language](https://ralph.alephium.org/)
