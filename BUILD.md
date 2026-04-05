# Building Alepic on Elementary OS

This guide provides step-by-step instructions for building the Alepic collaborative canvas application on Elementary OS (and other Ubuntu-based Linux distributions).

## Prerequisites

Elementary OS is based on Ubuntu, so most instructions work for both. These steps have been tested on Elementary OS 7.1 (Horus).

### 1. Update System Packages

```bash
sudo apt update && sudo apt upgrade -y
```

### 2. Install Build Dependencies

Alepic requires several system libraries for GUI rendering, networking, and security features:

```bash
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libfontconfig1-dev \
    libfreetype6-dev \
    libexpat1-dev \
    libdbus-1-dev \
    libudev-dev \
    libwayland-dev \
    libx11-dev \
    cmake \
    git \
    curl
```

### 3. Install Rust Toolchain

Alepic is written in Rust. Install the Rust toolchain using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

After installation, either restart your terminal or source the environment:

```bash
source $HOME/.cargo/env
```

Verify Rust installation:

```bash
rustc --version
cargo --version
```

You should see version numbers for both commands.

### 4. Install Node.js (Optional - for Smart Contract Tooling)

If you plan to compile or interact with the Alephium smart contracts:

```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs npm
```

### 5. Install Alephium Ralph Compiler (Optional)

To compile the smart contract `contracts/AlepicMain.ralph`:

```bash
# Clone the alephium project repository
git clone https://github.com/alephium/alephium.git
cd alephium

# Build the Ralph compiler (requires Scala/SBT)
# Alternatively, download pre-built binaries from:
# https://github.com/alephium/alephium/releases
```

Or use the web-based Ralph compiler at: https://ralph.alephium.org/

## Building the Application

### 1. Navigate to Project Directory

```bash
cd /path/to/alepic
```

### 2. Build in Release Mode

```bash
cargo build --release
```

This will download all dependencies and compile the application. The first build may take 5-10 minutes depending on your system.

### 3. Run the Application

After successful compilation:

```bash
cargo run --release
```

Or run the binary directly:

```bash
./target/release/alepic
```

## Build Troubleshooting

### Common Issues

#### Missing GTK3 Development Files

**Error:** `package gtk+-3.0 was not found`

**Solution:**
```bash
sudo apt install libgtk-3-dev
```

#### Missing OpenSSL Development Files

**Error:** `Could not find openssl via pkg-config`

**Solution:**
```bash
sudo apt install libssl-dev pkg-config
```

#### Missing XCB Libraries

**Error:** `could not find native static library `xcb_render`

**Solution:**
```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

#### Wayland Compatibility Issues

If you experience display issues on Wayland:

```bash
# Force X11 backend
export GDK_BACKEND=x11
cargo run --release
```

#### Low FPS or Rendering Issues

Ensure you have proper GPU drivers installed:

```bash
# For Intel GPUs
sudo apt install mesa-vulkan-drivers

# For AMD GPUs
sudo apt install mesa-vulkan-drivers vulkan-utils

# For NVIDIA GPUs
sudo apt install nvidia-driver-525 nvidia-settings
```

### Clean Build

If you encounter strange compilation errors, try a clean build:

```bash
cargo clean
cargo build --release
```

### Update Dependencies

To update all dependencies to their latest compatible versions:

```bash
cargo update
cargo build --release
```

## Running in Billboard Mode

Billboard mode provides a read-only public display:

```bash
./target/release/alepic --billboard
```

Or set the environment variable:

```bash
export ALEPIC_MODE=billboard
cargo run --release
```

## Simulation vs Real Mode

### Simulation Mode (Default)

The application starts in simulation mode by default, which:
- Mocks blockchain interactions
- Allows testing without real ALPH tokens
- Simulates Alepe jumps and auctions

### Real Mode

To connect to the actual Alephium network:

1. Edit the configuration in the application UI
2. Provide a valid Alephium node URL
3. Provide the deployed smart contract address
4. Connect your Alephium wallet

**Warning:** Real mode involves real cryptocurrency transactions. Use with caution!

## Smart Contract Deployment

### Compile the Contract

Using the Ralph compiler:

```bash
cd contracts
ralphc AlepicMain.ralph
```

This generates `.alph` bytecode files ready for deployment.

### Deploy to Alephium Network

1. Open Alephium Web Wallet (https://wallet.alephium.org/)
2. Navigate to the "Deploy Contract" section
3. Upload the compiled `.alph` file
4. Pay the deployment fee in ALPH
5. Note the contract address for client configuration

## Development Mode

For active development with faster iteration:

```bash
# Build in debug mode (faster compilation, slower execution)
cargo build

# Run with hot reloading (if supported)
cargo watch -x run
```

Install cargo-watch if needed:

```bash
cargo install cargo-watch
```

## Performance Optimization

### Build with Optimizations

The `--release` flag enables optimizations:

```bash
cargo build --release
```

### Profile-Guided Optimization (Advanced)

For maximum performance:

```bash
# Install cargo-pgo
cargo install cargo-pgo

# Build with PGO
cargo pgo build --release
```

## Cross-Compilation

To build for other Linux distributions or architectures:

```bash
# Example: Build for ARM64
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Run tests with output visible:

```bash
cargo test -- --nocapture
```

Run specific test modules:

```bash
# Canvas system tests
cargo test --test canvas_tests

# Alepe game mechanics tests
cargo test --test alepe_tests

# Fee system tests
cargo test --test fees_tests

# Content filter tests
cargo test --test content_filter_tests
```

### Test Coverage

The test suite covers:

**Canvas System (`tests/canvas_tests.rs`):**
- Chunk creation and initialization
- Pixel operations (set/get)
- Coordinate transformations (grid ↔ ID ↔ pixel)
- Canvas boundaries and edge cases
- Dirty chunk tracking
- Texture data generation

**Alepe Game (`tests/alepe_tests.rs`):**
- Alepe creation and initial positioning
- Jump timing and block intervals
- Movement distance validation
- Position calculations with offsets
- Chunk occupation logic (2x2 area)
- Auction chunk detection
- Canvas wrapping behavior
- Edge cases and boundary conditions

**Fee System (`tests/fees_tests.rs`):**
- Initial sale calculations (95% Treasury, 5% Referrer)
- Secondary sale calculations (95% Seller, 4% Treasury, 1% Referrer)
- Referrer fee handling (with/without)
- Rounding behavior
- Boundary values and edge cases
- Percentage accuracy verification

**Content Filter (`tests/content_filter_tests.rs`):**
- Basic filtering enable/disable
- Strict mode behavior
- Solid color detection (spam prevention)
- Flashing pattern detection
- Blocked pattern management
- Text-like pattern detection
- Moderation result types
- Advanced filter with neural network support

### Adding New Tests

To add new tests:

1. Create a new test file in `tests/` directory
2. Add test module configuration to `Cargo.toml`:
   ```toml
   [[test]]
   name = "your_test_name"
   path = "tests/your_test_name.rs"
   ```

3. Write tests following the existing patterns:
   ```rust
   #[cfg(test)]
   mod tests {
       use crate::your_module::YourStruct;
       
       #[test]
       fn test_your_feature() {
           // Test implementation
       }
   }
   ```

4. Run tests to verify:
   ```bash
   cargo test --test your_test_name
   ```

### Smart Contract Testing

**CRITICAL:** Smart contract testing requires separate tooling. See [TESTING.md](TESTING.md) for comprehensive security testing methodology.

```bash
# Install dependencies
npm install

# Run smart contract tests (requires local Alephium node or mock)
npm run test:contract

# Run with coverage
npm run test:coverage
```

Smart contract tests focus on:
- **Treasury Protection**: Preventing unauthorized withdrawals
- **Reentrancy Guards**: Preventing recursive call attacks
- **Fee Distribution Integrity**: Verifying exact percentage calculations
- **Access Control**: Ensuring only authorized functions can be called
- **Game Logic**: Validating Alepe jump mechanics and rewards

### Continuous Integration

For CI/CD pipelines, run:

```bash
# Run all tests with verbose output
cargo test --verbose

# Run tests in release mode
cargo test --release

# Generate test coverage report (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

See [TESTING.md](TESTING.md) for complete CI/CD configuration and security audit procedures.

## Documentation

Generate local documentation:

```bash
cargo doc --open
```

## Additional Resources

- [Rust Programming Language](https://doc.rust-lang.org/book/)
- [eframe/Egui Documentation](https://docs.rs/eframe/)
- [Alephium Documentation](https://docs.alephium.org/)
- [Ralph Language Guide](https://ralph.alephium.org/)

## Support

For issues specific to Elementary OS:
- [Elementary OS Forum](https://forum.elementary.io/)
- [AskUbuntu](https://askubuntu.com/)

For Alepic-specific issues:
- Check the CODE.md programmer manual
- Review the README.md design document
- Open an issue on the project repository
