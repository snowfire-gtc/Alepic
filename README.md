# Alepic

The Alephium Collaborative Canvas with the Alepe-the-Frog mascot.

*This is the most recent version of the Design Document written without any use of an AI.

## User Experience

User interacts with Smart Contract through official and 3rd party Clients.
Client Provider get Reward from every Chunk Buy/Sell operation.

1. Buy Pixel Chunks for ALPH.
2. Draw on your Pixel Chunks.
3. Sell Pixel Chunks for ALPH.
4. Get Rewards from the Treasury (the "Alepe game").
5. Display the Image on your Billboard.
6. View the Image on a city streets.
7. (Rent Pixels).
8. (???)
9. Alepe dominates the Universe.

## Rules

1. All Rules are built-in and performed by the Alepic Smart Contract.
2. User can pay N ALPH to buy a Random Unowned Chunk, where N=1 ALPH initially, after every Random Unowned Chunk purchase N increases by one: N=N+1.
3. User can take part in Auction to buy one or more of Unowned Chunks around the Alepe's position (Auction Chunks).
4. Alepe occupies a field of 2x2 Chunks.
5. User can change pixel colors on his own Chunks. After the User finished drawing, he pushes "Submit" button to save the pixels in blockchain history.
6. User can sell any of his own Chunks for ALPH.
7. Chunk size: 16x16 pixels.
8. Chunk colors: 4-bit color palette.

### Fees

| Case | Client Provider Reward | Treasury Reward | Seller Reward |
| ---- | --------------- | -------- | ------ |
| Unowned Chunk Sold | 5% | 95% | 0% |
| Owned Chunk Sold | 1% | 4% | 95% |

### The Alepe Game

Alepe the Frog is the Mascot of the Alepic project.

1. Randomly Jumps every 100,000 Alephium Blocks (about 27 hours).
2. Randomly Jumps on a distance ranged from 48 to 96 pixels left/right, up/down.
3. Owner of Chunk where Alepe lands gets 1% Reward of the Treasury.
4. Unowned Chunks around the place where the Alepe lands (Auction Chunks) could be bought via the Auction.

### Auctions

1. Every User can make a Stake for every Unowned Chunk plot around the Alepe position (the Auction Chunk).
2. 1,000 blocks before the next Alepe Jump, the Auction ends, and for every Auction Chunk, the User with the best Stake (Winner) pays his Stake according to the Fees table and owns the corresponding Unowned Chunk, all other Stakes go to the Treasury.

# Application Design

## Architecture

``` Diagram
Rust or TypeScript Client
^
|
v
TypeScript Bridge
^
|
v
Alephium Smart Contract <-> Alephium User Wallet
^
|
v
Treasury
```

## Storage

1. Chunk ownership is stored in the Smart Contract Memory.
2. Chunk data is stored in the Alephium Blockchain History.
3. The Alepe Game State is stored in the Smart Contract Memory.
4. User Wallet connection is stored in a secure way in a local key-ring.

## File Structure

``` Diagram
alepic-client/
├── Cargo.toml
├── src/
│	├── main.rs				# Entry point.
│	├── app.rs				# Application logic (State, Layers).
│	├── canvas.rs			# Canvas logic (Chunks, Grid, Reconstruction).
│	├── rendering.rs		# Wgpu/Texture management.
│	├── blockchain.rs		# Interaction with Alephium.
│	├── ui/					#
│	│	├── layers.rs		# Switch Layers (Color, Market).
│	│	├── widgets.rs		# Buttons, Palette, Dialogs.
│	│	└── inputs.rs		# Zoom, Pan handling.
│	└── utils.rs			# Utilities: 4-bit palette, coordinate systems, etc...
└── assets/					#
	├── icons/				# Icons: layers, connect wallet, save changed pixels.
	├── sounds/				# Sounds: Alepe jumps, Chunk changes, etc...
	└── fonts/				# Fonts.
```

# How to Build

## Build the Smart Contract

## Build the Client Application

``` bash
cargo build --release
```

## Install the required NPM packages

# Who is Alepe?

1. Alepe is a frog, but it is not a typical one frog.
2. She is a Princess, who owns the Treasury.
3. She gives to the Users her mercy whe she lands on their Chunks.

# Roadmap

Everything must be released as soon as possible.

# References

https://ralph.alephium.org/
https://rust-lang.org/
https://www.typescriptlang.org/
