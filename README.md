# Alepic

The Alephium Collaborative Canvas with the Alepe-the-Frog mascot.

*This is the most recent version of the Design Document written without any use of an AI.

## Constitution

Project's Values are:

1. Respect.
2. Decentralization.
3. User Safety.
4. Fair Access.
5. Fun.
6. Social Interaction.
7. Cultural Legacy.

## User Experience

User interacts with Smart Contract through official and 3rd party Clients.
Client Provider get Reward from every Chunk Buy/Sell operation.

1. Buy Pixel Chunks for ALPH.
2. Draw on your Pixel Chunks.
3. Sell Pixel Chunks for ALPH.
4. Get Rewards from the Treasury (the "Alepe game").
5. Display the Image on your Billboard.
6. View the Image on a city streets.
7. (Rent Mechanics).
8. (???)
9. Alepe dominates the Universe.

## Rules

1. All Rules are built-in and performed by the Alepic Smart Contract.
2. User can pay N ALPH to buy a Random Unowned Chunk, where N=1 ALPH initially, after every Random Unowned Chunk purchase N increases by one: N=N+1.
3. User can take part in Auction to buy one or more of Unowned Chunks around the Alepe's position (Auction Chunks).
4. Alepe occupies a field of 2x2 Chunks.
5. User can change pixel colors on his own Chunks. After the User finished drawing, he pushes "Submit" button to save the pixels in blockchain history.
6. User can sell any of his own Chunks for ALPH.
7. Chunk ownership is exclusive.
8. Chunk size: 16x16 pixels.
9. Chunk colors: 4-bit color palette.
10. Image Size: 4096 x 2160 pixels (UHD/4k) resulting in 34,560 Chunks (256×135).

### Fees

| Case | Client Provider Reward | Treasury Reward | Seller Reward |
| ---- | --------------- | -------- | ------ |
| Unowned Chunk Sold | 5% | 95% | 0% |
| Owned Chunk Sold | 1% | 4% | 95% |

### The Alepe Game

Alepe the Frog is the Mascot of the Alepic project.

1. Randomly Jumps every 100,000 Alephium Blocks (about 27 hours 46 minutes).
2. Randomly Jumps on a distance ranged from 48 to 96 pixels left/right, up/down.
3. Owner of Chunk where Alepe lands gets 1% Reward of the Treasury.
4. Unowned Chunks around the place where the Alepe lands (Auction Chunks) could be bought via the Auction.

Alepe's jump is described by the following pseudocode:

``` Pseudocode
dx = random(0 … MAX_DISTANCE-MIN_DISTANCE) + MIN_DISTANCE
dy = random(0 … MAX_DISTANCE-MIN_DISTANCE) + MIN_DISTANCE
sign_x = random(0..1)
sign_y = random(0..1)
x = sign_x>0 ? (x+dx)%WIDTH : (x+WIDTH-dx)%WIDTH
y = sign_y>0 ? (y+dy)%HEIGHT : (y+HEIGHT-dy)%HEIGHT
```

### The Auction

1. Every User can make a Stake for every Unowned Chunk plot around the Alepe position (the Auction Chunk).
2. 1,000 blocks before the next Alepe Jump, the Auction ends, and for every Auction Chunk, the User with the best Stake (Winner) pays his Stake according to the Fees table and owns the corresponding Unowned Chunk, all other Stakes go to the Treasury.
3. Minimal Stake: 1 ALPH. Maximal Stake: Unlimited.
4. If there are two or more equal Stakes, then the Winner is defined by a random choice (Smart Contract Rule).

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
2. Chunk color data is stored in the Alephium Blockchain History.
3. Chunk price is stored in the Smart Contract Memory.
4. The Alepe Game State is stored in the Smart Contract Memory.
5. Next Random Unowned Chunk price is stored in the Smart Contract Memory.
6. User Wallet connection is stored in a secure way in a local key-ring.

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

## UI

* All area of Application is occupied by a 2D Image View which displays all visible chunks. Every chunk is stored as a texture which updates only after at least one of its pixels changes its color.
* The Client Application implements mechanics to protect the User from failed Transactions.

### General User interactions

* Zoom: mouse wheel, or pageUp/pageDown.
* Pan: left mouse button click&drag, or arrow keys.
* Switch Layer: over the image, in the bottom left corner of the Image View, the button with layers icon is shown. When the User clicks this button, View switches to the next View Layer.

### View Layers

Every View Layer adds additional custom interactions and UI elements to the basic Image View.

#### Color Layer

* **Image**, cached from the Chunks obtained from the Alephium block history.
* **Color Palette**: show palette to choose pixel color. The Palette is presented by a set of square colored tiles densely packed in two rows (first row: bright colors, second row: dark colors) located in the horizontal panel located in the middle of the bottom part of the View.
* Button with a **Submit** label, located in the bottom right corner of the View.
* **Choose Color** Interaction: left-click on palette.
* **Draw** Interaction: left mouse click on owned chunk. Altered pixels are stored locally only.
* **Submit** Interaction: User clicks the **Submit** button and Save Dialog window appears, where User can check fees and confirm the Transaction.

#### Market Layer:

* Above every Unowned Chunk (excluding Auction Chunks) and every Chunk being sold - a text label is shown indicating the corresponding Chunk's price.
* Auction Chunks have a special color.
* Button with a **wallet icon**, located in the bottom right corner.
* **Wallet Connect** Interaction: User clicks on the Wallet Button and Applications shows the Wallet Connect dialog.
* **Sell** Interaction: User clicks on the one of his Chunks with the left mouse button and Application shows the Sell Dialog where the User can set the Chunk Sell Price, checks for fees and approves the message to the smart contract.
* **Buy** Interaction: User clicks on the Chunk not owned by him. Application show Buy Dialog Window where user can check Chunk price and fees and confirm transaction.
* **Stake** Interaction: User clicks on one of Auction Chunks and dialog appears where the User can submit his Stake, check fees and read the Auction Rules.

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

# Technical Information

1. Transaction Limit: 32 KB per block is strictly enforced. Updates are batched or
fragmented across consecutive blocks if a chunk update exceeds single-block capacity.
2. Smart Contract Execution cost.
3. Smart Contract Memory limitations.
4. User Safety Considerations.

# Inappropriate Content Management.

Client Providers take responsibility for inappropriate content demonstration. They can use neural networks to filter out all inappropriate content in a sensible jurisdictions.

# Roadmap

Everything must be released as soon as possible.

# References

1. https://ralph.alephium.org/
2. https://rust-lang.org/
3. https://www.typescriptlang.org/
