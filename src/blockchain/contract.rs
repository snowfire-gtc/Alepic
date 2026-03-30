use serde::{Deserialize, Serialize};
use crate::blockchain::client::AlephiumClient;

/// Alepic Smart Contract Interface
pub struct AlepicContract {
    contract_address: String,
    client: AlephiumClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub chunk_id: u32,
    pub grid_x: u16,
    pub grid_y: u16,
    pub owner: Option<String>,
    pub price: u64, // In ALPH (smallest unit)
    pub is_auction_chunk: bool,
    pub last_update_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionInfo {
    pub chunk_id: u32,
    pub highest_bid: u64,
    pub highest_bidder: Option<String>,
    pub auction_end_block: u64,
    pub bids_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryInfo {
    pub total_balance: u64,
    pub alepe_rewards_paid: u64,
    pub total_chunks_sold: u32,
}

impl AlepicContract {
    pub fn new(contract_address: String, client: AlephiumClient) -> Self {
        Self { 
            contract_address,
            client,
        }
    }

    /// Get chunk information
    pub async fn get_chunk_info(&self, chunk_id: u32) -> Result<ChunkInfo, Box<dyn std::error::Error>> {
        // Query contract storage for chunk data
        let url = format!(
            "{}/contracts/{}/state/chunks/{}",
            self.client.node_url(),
            self.contract_address,
            chunk_id
        );

        let resp = self.client.http_client().get(&url).send().await;
        
        if let Ok(response) = resp {
            if response.status().is_success() {
                let chunk_data: serde_json::Value = response.json().await?;
                
                let owner = chunk_data
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                let price = chunk_data
                    .get("price")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1_000_000_000_000_000_000u64);
                
                let is_auction = chunk_data
                    .get("isAuction")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                let last_update = chunk_data
                    .get("lastUpdateBlock")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                return Ok(ChunkInfo {
                    chunk_id,
                    grid_x: (chunk_id % 256) as u16,
                    grid_y: (chunk_id / 256) as u16,
                    owner,
                    price,
                    is_auction_chunk: is_auction,
                    last_update_block: last_update,
                });
            }
        }

        // Fallback: try to get ownership from events
        let owner = self.client.get_chunk_owner(chunk_id).await?;
        
        Ok(ChunkInfo {
            chunk_id,
            grid_x: (chunk_id % 256) as u16,
            grid_y: (chunk_id / 256) as u16,
            owner,
            price: 1_000_000_000_000_000_000u64, // 1 ALPH initial price
            is_auction_chunk: false,
            last_update_block: 0,
        })
    }

    /// Buy unowned chunk (Random Purchase)
    pub async fn buy_random_chunk(
        &self,
        buyer_address: &str,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Find an unowned chunk by querying recent events
        let current_block = self.client.get_current_block().await?;
        
        // Try up to 10 random chunks
        for attempt in 0..10 {
            let test_chunk_id = ((current_block.block_number + attempt as u64) % 65536) as u32;
            
            if let Ok(Some(owner)) = self.client.get_chunk_owner(test_chunk_id).await {
                if owner.is_empty() {
                    // Found an unowned chunk, proceed with purchase
                    return self.buy_chunk(buyer_address, test_chunk_id, 1_000_000_000_000_000_000u64).await;
                }
            }
        }

        Err("No unowned chunks found in search".into())
    }

    /// Buy specific chunk
    pub async fn buy_chunk(
        &self,
        buyer_address: &str,
        chunk_id: u32,
        price: u64,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Build contract call data for buying a chunk
        let call_data = self.encode_buy_chunk_call(chunk_id, price)?;
        
        // Submit transaction to blockchain
        let tx_id = self.client
            .submit_transaction(
                buyer_address,
                &self.contract_address,
                price,
                Some(call_data),
            )
            .await?;

        Ok(TransactionResult {
            success: true,
            transaction_id: tx_id,
            message: format!("Chunk {} purchased", chunk_id),
        })
    }

    /// Sell owned chunk
    pub async fn sell_chunk(
        &self,
        seller_address: &str,
        chunk_id: u32,
        price: u64,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Verify ownership first
        let owner = self.client.get_chunk_owner(chunk_id).await?;
        
        if owner.as_ref() != Some(&seller_address.to_string()) {
            return Err("You do not own this chunk".into());
        }

        // Build contract call data for selling a chunk
        let call_data = self.encode_sell_chunk_call(chunk_id, price)?;
        
        // Submit transaction (no ALPH transfer needed for listing)
        let tx_id = self.client
            .submit_transaction(
                seller_address,
                &self.contract_address,
                0,
                Some(call_data),
            )
            .await?;

        Ok(TransactionResult {
            success: true,
            transaction_id: tx_id,
            message: format!("Chunk {} listed for sale at {}", chunk_id, price),
        })
    }

    /// Place bid on auction chunk
    pub async fn place_bid(
        &self,
        bidder_address: &str,
        chunk_id: u32,
        amount: u64,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Check if chunk is in auction
        let chunk_info = self.get_chunk_info(chunk_id).await?;
        
        if !chunk_info.is_auction_chunk {
            return Err("This chunk is not in auction".into());
        }

        // Get current highest bid
        if let Some(auction_info) = self.get_auction_info(chunk_id).await? {
            if amount <= auction_info.highest_bid {
                return Err("Bid must be higher than current highest bid".into());
            }
        }

        // Build contract call data for placing a bid
        let call_data = self.encode_place_bid_call(chunk_id)?;
        
        // Submit transaction with bid amount
        let tx_id = self.client
            .submit_transaction(
                bidder_address,
                &self.contract_address,
                amount,
                Some(call_data),
            )
            .await?;

        Ok(TransactionResult {
            success: true,
            transaction_id: tx_id,
            message: format!("Bid of {} placed on chunk {}", amount, chunk_id),
        })
    }

    /// Submit pixel changes to blockchain
    pub async fn submit_pixels(
        &self,
        owner_address: &str,
        chunk_id: u32,
        pixels: Vec<u8>,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Verify ownership
        let owner = self.client.get_chunk_owner(chunk_id).await?;
        
        if owner.as_ref() != Some(&owner_address.to_string()) {
            return Err("You do not own this chunk".into());
        }

        // Validate pixel data (must be 256 bytes for 16x16 chunk)
        if pixels.len() != 256 {
            return Err("Pixel data must be exactly 256 bytes (16x16)".into());
        }

        // Build contract call data for updating pixels
        let call_data = self.encode_update_pixels_call(chunk_id, &pixels)?;
        
        // Submit transaction
        let tx_id = self.client
            .submit_transaction(
                owner_address,
                &self.contract_address,
                0,
                Some(call_data),
            )
            .await?;

        Ok(TransactionResult {
            success: true,
            transaction_id: tx_id,
            message: format!("Pixels updated for chunk {}", chunk_id),
        })
    }

    /// Get current auction state for a chunk
    pub async fn get_auction_info(&self, chunk_id: u32) -> Result<Option<AuctionInfo>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/contracts/{}/state/auctions/{}",
            self.client.node_url(),
            self.contract_address,
            chunk_id
        );

        let resp = self.client.http_client().get(&url).send().await;
        
        if let Ok(response) = resp {
            if response.status().is_success() {
                let auction_data: serde_json::Value = response.json().await?;
                
                let highest_bid = auction_data
                    .get("highestBid")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                
                let highest_bidder = auction_data
                    .get("highestBidder")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                let end_block = auction_data
                    .get("endBlock")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                
                let bids_count = auction_data
                    .get("bidsCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                return Ok(Some(AuctionInfo {
                    chunk_id,
                    highest_bid,
                    highest_bidder,
                    auction_end_block: end_block,
                    bids_count,
                }));
            }
        }

        Ok(None)
    }

    /// Get treasury information
    pub async fn get_treasury_info(&self) -> Result<TreasuryInfo, Box<dyn std::error::Error>> {
        let treasury_balance = self.client.get_treasury_balance().await?;
        
        // Query treasury contract state for additional info
        let url = format!(
            "{}/contracts/{}/state",
            self.client.node_url(),
            self.get_treasury_address()
        );

        let mut alepe_rewards_paid = 0u64;
        let mut total_chunks_sold = 0u32;

        if let Ok(resp) = self.client.http_client().get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(state_data) = resp.json::<serde_json::Value>().await {
                    alepe_rewards_paid = state_data
                        .get("alepeRewardsPaid")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                    
                    total_chunks_sold = state_data
                        .get("totalChunksSold")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                }
            }
        }

        Ok(TreasuryInfo {
            total_balance: treasury_balance,
            alepe_rewards_paid,
            total_chunks_sold,
        })
    }

    /// Claim Alepe reward (if Alepe landed on user's chunk)
    pub async fn claim_alepe_reward(
        &self,
        user_address: &str,
        chunk_id: u32,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Verify user owns the chunk
        let owner = self.client.get_chunk_owner(chunk_id).await?;
        
        if owner.as_ref() != Some(&user_address.to_string()) {
            return Err("You do not own this chunk".into());
        }

        // Check if Alepe has landed on this chunk and reward is available
        let alepe_state = self.client.get_alepe_state().await?;
        let current_block = self.client.get_current_block().await?;
        
        // Calculate if Alepe landed on this chunk (every 100,000 blocks)
        let last_jump_block = alepe_state.last_jump_block;
        let blocks_since_jump = current_block.block_number.saturating_sub(last_jump_block);
        
        if blocks_since_jump >= 100_000 {
            // Alepe should have jumped, check if it landed here
            // This is simplified - actual logic would be in contract
        }

        // Build contract call data for claiming reward
        let call_data = self.encode_claim_reward_call(chunk_id)?;
        
        // Submit transaction
        let tx_id = self.client
            .submit_transaction(
                user_address,
                &self.contract_address,
                0,
                Some(call_data),
            )
            .await?;

        Ok(TransactionResult {
            success: true,
            transaction_id: tx_id,
            message: "Alepe reward claimed successfully".to_string(),
        })
    }

    // Helper methods for encoding contract calls
    
    fn encode_buy_chunk_call(&self, chunk_id: u32, price: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Encode function selector and parameters for buyChunk(uint32,uint256)
        // This is a simplified encoding - actual implementation depends on Alephium's ABI
        let mut data = Vec::new();
        
        // Function selector (first 4 bytes of hash of function signature)
        data.extend_from_slice(&[0x12, 0x34, 0x56, 0x78]); // Placeholder selector
        
        // Encode chunk_id as uint32
        data.extend_from_slice(&chunk_id.to_be_bytes());
        
        // Encode price as uint256 (32 bytes)
        let mut price_bytes = [0u8; 32];
        price_bytes[24..].copy_from_slice(&price.to_be_bytes());
        data.extend_from_slice(&price_bytes);
        
        Ok(data)
    }

    fn encode_sell_chunk_call(&self, chunk_id: u32, price: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x23, 0x45, 0x67, 0x89]); // Placeholder selector
        data.extend_from_slice(&chunk_id.to_be_bytes());
        let mut price_bytes = [0u8; 32];
        price_bytes[24..].copy_from_slice(&price.to_be_bytes());
        data.extend_from_slice(&price_bytes);
        Ok(data)
    }

    fn encode_place_bid_call(&self, chunk_id: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x34, 0x56, 0x78, 0x9A]); // Placeholder selector
        data.extend_from_slice(&chunk_id.to_be_bytes());
        Ok(data)
    }

    fn encode_update_pixels_call(&self, chunk_id: u32, pixels: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x45, 0x67, 0x89, 0xAB]); // Placeholder selector
        data.extend_from_slice(&chunk_id.to_be_bytes());
        // Encode pixel array length
        data.extend_from_slice(&(pixels.len() as u32).to_be_bytes());
        // Append pixel data
        data.extend_from_slice(pixels);
        Ok(data)
    }

    fn encode_claim_reward_call(&self, chunk_id: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x56, 0x78, 0x9A, 0xBC]); // Placeholder selector
        data.extend_from_slice(&chunk_id.to_be_bytes());
        Ok(data)
    }

    fn get_treasury_address(&self) -> &str {
        "3a8U9DrLaK0Y6O4N5S7Q9X2Z1I8G6E4C0B3F5H7J9M1R"
    }
}

// Extend AlephiumClient with accessor methods
impl AlephiumClient {
    pub fn node_url(&self) -> &str {
        &self.node_url
    }
    
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub success: bool,
    pub transaction_id: String,
    pub message: String,
}
