use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Alephium RPC Client for interacting with the blockchain
pub struct AlephiumClient {
    client: Client,
    node_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub block_number: u64,
    pub timestamp: u64,
    pub transactions_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    pub address: String,
    pub balance: u64, // In ALPH (smallest unit)
    pub locked_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RpcResponse<T> {
    result: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockHeader {
    chain_from: u8,
    chain_to: u8,
    height: u64,
    timestamp: u64,
    deps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BalanceResponse {
    balance: String,
    locked_balance: String,
}

impl AlephiumClient {
    pub fn new(node_url: String) -> Self {
        Self {
            client: Client::new(),
            node_url,
        }
    }

    /// Get current block number
    pub async fn get_current_block(&self) -> Result<BlockInfo, Box<dyn std::error::Error>> {
        // Fetch the latest block header from all chains
        let mut max_height = 0u64;
        let mut latest_timestamp = 0u64;
        let mut total_txs = 0u32;

        // Alephium has multiple chains (typically 3), we check each
        for chain_from in 0..3 {
            for chain_to in 0..3 {
                let url = format!(
                    "{}/infos/blocks/latest?fromGroup={}&toGroup={}",
                    self.node_url, chain_from, chain_to
                );
                
                if let Ok(resp) = self.client.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(block_data) = resp.json::<BlockHeader>().await {
                            if block_data.height > max_height {
                                max_height = block_data.height;
                                latest_timestamp = block_data.timestamp;
                            }
                        }
                    }
                }
            }
        }

        // Get transaction count from recent blocks
        let txs_url = format!("{}/infos/transactions/count", self.node_url);
        if let Ok(resp) = self.client.get(&txs_url).send().await {
            if resp.status().is_success() {
                if let Ok(tx_data) = resp.json::<serde_json::Value>().await {
                    total_txs = tx_data.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                }
            }
        }

        Ok(BlockInfo {
            block_number: max_height,
            timestamp: latest_timestamp,
            transactions_count: total_txs,
        })
    }

    /// Get account balance
    pub async fn get_balance(&self, address: &str) -> Result<AccountBalance, Box<dyn std::error::Error>> {
        let url = format!("{}/addresses/{}/balance", self.node_url, address);
        
        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            return Err(format!("Failed to get balance: {}", resp.status()).into());
        }

        let balance_data: BalanceResponse = resp.json().await?;
        
        // Parse balance strings to u64 (Alephium returns balances as strings)
        let balance = balance_data.balance.parse::<u64>().unwrap_or(0);
        let locked_balance = balance_data.locked_balance.parse::<u64>().unwrap_or(0);

        Ok(AccountBalance {
            address: address.to_string(),
            balance,
            locked_balance,
        })
    }

    /// Submit a transaction to the blockchain
    pub async fn submit_transaction(
        &self,
        from_address: &str,
        to_address: &str,
        amount: u64,
        data: Option<Vec<u8>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/transactions/build", self.node_url);
        
        // Build transaction payload
        let mut payload = json!({
            "fromAddress": from_address,
            "toAddress": to_address,
            "amount": amount.to_string(),
        });

        if let Some(tx_data) = data {
            // Convert data to hex string for contract call
            let hex_data = hex::encode(&tx_data);
            payload["data"] = json!(hex_data);
        }

        let resp = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(format!("Transaction build failed: {}", error_text).into());
        }

        let tx_result: serde_json::Value = resp.json().await?;
        let tx_id = tx_result
            .get("txId")
            .and_then(|v| v.as_str())
            .ok_or("No transaction ID returned")?
            .to_string();

        // Submit the signed transaction
        let submit_url = format!("{}/transactions/send", self.node_url);
        let submit_payload = json!({
            "txId": tx_id,
            "unsignedTx": tx_result.get("unsignedTx").cloned().unwrap_or_default(),
        });

        let submit_resp = self.client
            .post(&submit_url)
            .json(&submit_payload)
            .send()
            .await?;

        if !submit_resp.status().is_success() {
            let error_text = submit_resp.text().await?;
            return Err(format!("Transaction send failed: {}", error_text).into());
        }

        Ok(tx_id)
    }

    /// Get chunk ownership from blockchain history
    pub async fn get_chunk_owner(&self, chunk_id: u32) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Query events from the Alepic contract for ChunkPurchased events
        let contract_address = self.get_alepic_contract_address();
        let url = format!(
            "{}/events?contractAddress={}&eventName=ChunkPurchased&fromBlock={}",
            self.node_url,
            contract_address,
            0 // Could optimize by starting from a specific block
        );

        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            return Ok(None);
        }

        let events: Vec<serde_json::Value> = resp.json().await?;
        
        // Find the most recent event for this chunk_id
        let mut latest_owner: Option<String> = None;
        let mut latest_block = 0u64;

        for event in events {
            if let Some(chunk) = event.get("chunkId").and_then(|v| v.as_u64()) {
                if chunk == chunk_id as u64 {
                    if let Some(block_num) = event.get("blockNumber").and_then(|v| v.as_u64()) {
                        if block_num >= latest_block {
                            latest_block = block_num;
                            latest_owner = event
                                .get("buyer")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                        }
                    }
                }
            }
        }

        Ok(latest_owner)
    }

    /// Get chunk pixel data from blockchain history
    pub async fn get_chunk_pixels(&self, chunk_id: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Query events for PixelUpdate events
        let contract_address = self.get_alepic_contract_address();
        let url = format!(
            "{}/events?contractAddress={}&eventName=PixelUpdate&fromBlock={}",
            self.node_url,
            contract_address,
            0
        );

        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            return Ok(vec![0; 256]); // Return default empty pixels
        }

        let events: Vec<serde_json::Value> = resp.json().await?;
        
        // Initialize with default pixels (all black)
        let mut pixels = vec![0u8; 256]; // 16x16 = 256 pixels
        
        // Find the most recent pixel update for this chunk
        let mut latest_update_block = 0u64;

        for event in events {
            if let Some(chunk) = event.get("chunkId").and_then(|v| v.as_u64()) {
                if chunk == chunk_id as u64 {
                    if let Some(block_num) = event.get("blockNumber").and_then(|v| v.as_u64()) {
                        if block_num >= latest_update_block {
                            latest_update_block = block_num;
                            
                            // Extract pixel data from event
                            if let Some(pixel_data) = event.get("pixels").and_then(|v| v.as_array()) {
                                for (i, pixel) in pixel_data.iter().enumerate() {
                                    if i < 256 {
                                        pixels[i] = pixel.as_u64().unwrap_or(0) as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(pixels)
    }

    /// Get treasury balance
    pub async fn get_treasury_balance(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let treasury_address = self.get_treasury_address();
        let url = format!("{}/addresses/{}/balance", self.node_url, treasury_address);
        
        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            return Ok(0);
        }

        let balance_data: BalanceResponse = resp.json().await?;
        let balance = balance_data.balance.parse::<u64>().unwrap_or(0);

        Ok(balance)
    }

    /// Get Alepe game state from contract
    pub async fn get_alepe_state(&self) -> Result<AlepeState, Box<dyn std::error::Error>> {
        // Query the Alepe contract state
        let alepe_contract = self.get_alepe_contract_address();
        let url = format!(
            "{}/contracts/{}/state",
            self.node_url,
            alepe_contract
        );

        let resp = self.client.get(&url).send().await?;
        
        if !resp.status().is_success() {
            // Return default state if contract not found or error
            return Ok(AlepeState {
                grid_x: 128,
                grid_y: 67,
                last_jump_block: 0,
            });
        }

        let state: serde_json::Value = resp.json().await?;
        
        let grid_x = state
            .get("gridX")
            .and_then(|v| v.as_u64())
            .unwrap_or(128) as u16;
        let grid_y = state
            .get("gridY")
            .and_then(|v| v.as_u64())
            .unwrap_or(67) as u16;
        let last_jump_block = state
            .get("lastJumpBlock")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(AlepeState {
            grid_x,
            grid_y,
            last_jump_block,
        })
    }

    /// Helper to get Alepic contract address (should be configurable)
    fn get_alepic_contract_address(&self) -> &str {
        // This should be loaded from config or environment
        "2z7T8CqkZJ9X5N3M4R6P8W1Y0H7F5D3A9B2E4G6K8L0Q"
    }

    /// Helper to get treasury address
    fn get_treasury_address(&self) -> &str {
        // Treasury address derived from contract
        "3a8U9DrLaK0Y6O4N5S7Q9X2Z1I8G6E4C0B3F5H7J9M1R"
    }

    /// Helper to get Alepe contract address
    fn get_alepe_contract_address(&self) -> &str {
        // Alepe game contract address
        "4b9V0EsMbL1Z7P5O6T8R0Y3A2J9H7F5D1C4G6I8K0N2S"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlepeState {
    pub grid_x: u16,
    pub grid_y: u16,
    pub last_jump_block: u64,
}
