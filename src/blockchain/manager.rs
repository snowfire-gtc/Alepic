use crate::blockchain::client::AlephiumClient;
use crate::blockchain::contract::{AlepicContract, ChunkInfo, AuctionInfo, TreasuryInfo, TransactionResult};
use crate::blockchain::transactions::{TransactionData, TransactionStatus, TransactionType};
use crate::mode::OperationMode;
use std::collections::HashMap;

/// Hybrid blockchain manager supporting both Real and Simulation modes
pub struct BlockchainManager {
    mode: OperationMode,
    client: Option<AlephiumClient>,
    contract: Option<AlepicContract>,
    
    // Simulation state
    sim_chunks: HashMap<u32, ChunkInfo>,
    sim_balances: HashMap<String, u64>,
    sim_treasury_balance: u64,
    sim_current_block: u64,
    sim_transaction_counter: u64,
}

impl BlockchainManager {
    pub fn new(mode: OperationMode) -> Self {
        Self {
            mode,
            client: None,
            contract: None,
            sim_chunks: HashMap::new(),
            sim_balances: HashMap::new(),
            sim_treasury_balance: 1_000_000_000_000_000_000u64, // 1 ALPH initial treasury
            sim_current_block: 1_000_000,
            sim_transaction_counter: 0,
        }
    }

    /// Initialize blockchain connection (only in Real mode)
    pub fn init(&mut self, node_url: String, contract_address: String) {
        if self.mode.is_real() {
            self.client = Some(AlephiumClient::new(node_url));
            self.contract = Some(AlepicContract::new(contract_address));
        }
    }

    /// Get current operation mode
    pub fn get_mode(&self) -> OperationMode {
        self.mode
    }

    /// Switch operation mode
    pub fn set_mode(&mut self, mode: OperationMode) {
        self.mode = mode;
    }

    /// Get current block number
    pub async fn get_current_block(&self) -> Result<u64, Box<dyn std::error::Error>> {
        if self.mode.is_real() {
            if let Some(client) = &self.client {
                let block_info = client.get_current_block().await?;
                return Ok(block_info.block_number);
            }
        }
        // Simulation mode or fallback
        Ok(self.sim_current_block)
    }

    /// Get chunk information
    pub async fn get_chunk_info(&self, chunk_id: u32) -> Result<ChunkInfo, Box<dyn std::error::Error>> {
        if self.mode.is_real() {
            if let Some(contract) = &self.contract {
                return contract.get_chunk_info(chunk_id).await;
            }
        }
        
        // Simulation mode
        Ok(self.sim_chunks.get(&chunk_id).cloned().unwrap_or_else(|| {
            ChunkInfo {
                chunk_id,
                grid_x: chunk_id % 256,
                grid_y: chunk_id / 256,
                owner: None,
                price: 1_000_000_000_000_000_000u64,
                is_auction_chunk: false,
                last_update_block: self.sim_current_block,
            }
        }))
    }

    /// Get account balance
    pub async fn get_balance(&self, address: &str) -> Result<u64, Box<dyn std::error::Error>> {
        if self.mode.is_real() {
            if let Some(client) = &self.client {
                let account_info = client.get_balance(address).await?;
                return Ok(account_info.balance);
            }
        }
        
        // Simulation mode
        Ok(*self.sim_balances.get(address).unwrap_or(&10_000_000_000_000_000_000u64)) // 10 ALPH default
    }

    /// Submit transaction with retry logic and protection
    pub async fn submit_transaction_with_protection(
        &mut self,
        tx_data: TransactionData,
        max_retries: u32,
    ) -> Result<TransactionResult, TransactionError> {
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match self.execute_transaction(&tx_data).await {
                Ok(result) => {
                    if result.success {
                        return Ok(result);
                    } else {
                        return Err(TransactionError::TransactionFailed(result.message));
                    }
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // Wait before retry (exponential backoff)
                    if attempt < max_retries - 1 {
                        let delay_ms = 100 * (2u64.pow(attempt as u32));
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(TransactionError::UnknownError))
    }

    /// Execute a single transaction
    async fn execute_transaction(&self, tx_data: &TransactionData) -> Result<TransactionResult, TransactionError> {
        if self.mode.is_real() {
            if let Some(contract) = &self.contract {
                return match &tx_data.tx_type {
                    TransactionType::BuyChunk { chunk_id, price } => {
                        contract.buy_chunk(&tx_data.from, *chunk_id, *price).await
                            .map_err(|e| TransactionError::BlockchainError(e.to_string()))
                    }
                    TransactionType::SellChunk { chunk_id, price } => {
                        contract.sell_chunk(&tx_data.from, *chunk_id, *price).await
                            .map_err(|e| TransactionError::BlockchainError(e.to_string()))
                    }
                    TransactionType::PlaceBid { chunk_id, amount } => {
                        contract.place_bid(&tx_data.from, *chunk_id, *amount).await
                            .map_err(|e| TransactionError::BlockchainError(e.to_string()))
                    }
                    TransactionType::SubmitPixels { chunk_id, pixels } => {
                        // In real mode, batch if needed
                        contract.submit_pixels(&tx_data.from, *chunk_id, tx_data.data.clone()).await
                            .map_err(|e| TransactionError::BlockchainError(e.to_string()))
                    }
                    TransactionType::ClaimReward { chunk_id } => {
                        contract.claim_alepe_reward(&tx_data.from, *chunk_id).await
                            .map_err(|e| TransactionError::BlockchainError(e.to_string()))
                    }
                };
            }
        }
        
        // Simulation mode
        self.simulate_transaction(tx_data).await
    }

    /// Simulate transaction execution
    async fn simulate_transaction(&self, tx_data: &TransactionData) -> Result<TransactionResult, TransactionError> {
        // Check balance in simulation
        let required_balance = tx_data.value + (tx_data.gas_price * tx_data.gas_amount);
        let user_balance = self.sim_balances.get(&tx_data.from).copied().unwrap_or(10_000_000_000_000_000_000u64);
        
        if user_balance < required_balance {
            return Err(TransactionError::InsufficientBalance {
                required: required_balance,
                available: user_balance,
            });
        }

        // Generate mock transaction ID
        let tx_id = format!("sim_tx_{}", self.sim_transaction_counter);
        
        Ok(TransactionResult {
            success: true,
            transaction_id: tx_id,
            message: format!("Simulated {:?}", tx_data.tx_type),
        })
    }

    /// Get treasury info
    pub async fn get_treasury_info(&self) -> Result<TreasuryInfo, Box<dyn std::error::Error>> {
        if self.mode.is_real() {
            if let Some(contract) = &self.contract {
                return contract.get_treasury_info().await;
            }
        }
        
        Ok(TreasuryInfo {
            total_balance: self.sim_treasury_balance,
            alepe_rewards_paid: 0,
            total_chunks_sold: 0,
        })
    }

    /// Get auction info
    pub async fn get_auction_info(&self, chunk_id: u32) -> Result<Option<AuctionInfo>, Box<dyn std::error::Error>> {
        if self.mode.is_real() {
            if let Some(contract) = &self.contract {
                return contract.get_auction_info(chunk_id).await;
            }
        }
        
        Ok(None) // No auctions in simulation by default
    }

    /// Update simulation state (for testing)
    pub fn sim_set_balance(&mut self, address: String, balance: u64) {
        self.sim_balances.insert(address, balance);
    }

    pub fn sim_set_chunk_owner(&mut self, chunk_id: u32, owner: Option<String>, price: u64) {
        let chunk_info = self.sim_chunks.entry(chunk_id).or_insert_with(|| {
            ChunkInfo {
                chunk_id,
                grid_x: chunk_id % 256,
                grid_y: chunk_id / 256,
                owner: None,
                price: 1_000_000_000_000_000_000u64,
                is_auction_chunk: false,
                last_update_block: self.sim_current_block,
            }
        });
        chunk_info.owner = owner;
        chunk_info.price = price;
    }

    pub fn sim_advance_block(&mut self, blocks: u64) {
        self.sim_current_block += blocks;
    }
}

/// Transaction error types for protection against failures
#[derive(Debug, Clone)]
pub enum TransactionError {
    InsufficientBalance { required: u64, available: u64 },
    ChunkNotOwned,
    ChunkAlreadyOwned,
    InvalidPrice,
    AuctionEnded,
    TransactionFailed(String),
    BlockchainError(String),
    NetworkTimeout,
    GasPriceTooLow,
    UnknownError,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: required {} ALPH, available {} ALPH", 
                       required as f64 / 1e18, available as f64 / 1e18)
            }
            TransactionError::ChunkNotOwned => write!(f, "You don't own this chunk"),
            TransactionError::ChunkAlreadyOwned => write!(f, "Chunk is already owned"),
            TransactionError::InvalidPrice => write!(f, "Invalid price"),
            TransactionError::AuctionEnded => write!(f, "Auction has ended"),
            TransactionError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            TransactionError::BlockchainError(msg) => write!(f, "Blockchain error: {}", msg),
            TransactionError::NetworkTimeout => write!(f, "Network timeout"),
            TransactionError::GasPriceTooLow => write!(f, "Gas price too low"),
            TransactionError::UnknownError => write!(f, "Unknown error"),
        }
    }
}

impl std::error::Error for TransactionError {}

impl Default for BlockchainManager {
    fn default() -> Self {
        Self::new(OperationMode::Simulation)
    }
}
