use serde::{Deserialize, Serialize};

/// Transaction status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Submitted(String), // Transaction ID
    Confirmed(u64),    // Block number
    Failed(String),    // Error message
}

/// Transaction types supported by Alepic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    BuyChunk { chunk_id: u32, price: u64 },
    SellChunk { chunk_id: u32, price: u64 },
    PlaceBid { chunk_id: u32, amount: u64 },
    SubmitPixels { chunk_id: u32, pixel_count: u32 },
    ClaimReward { chunk_id: u32 },
}

/// Transaction builder for creating blockchain transactions
pub struct TransactionBuilder {
    from_address: String,
    gas_price: u64,
    gas_amount: u64,
}

impl TransactionBuilder {
    pub fn new(from_address: String) -> Self {
        Self {
            from_address,
            gas_price: 1_000_000_000u64, // Default gas price
            gas_amount: 100_000u64,       // Default gas amount
        }
    }

    pub fn with_gas(mut self, price: u64, amount: u64) -> Self {
        self.gas_price = price;
        self.gas_amount = amount;
        self
    }

    /// Build a buy chunk transaction
    pub fn build_buy(&self, chunk_id: u32, price: u64) -> TransactionData {
        TransactionData {
            tx_type: TransactionType::BuyChunk { chunk_id, price },
            from: self.from_address.clone(),
            value: price,
            gas_price: self.gas_price,
            gas_amount: self.gas_amount,
            data: Vec::new(),
        }
    }

    /// Build a sell chunk transaction
    pub fn build_sell(&self, chunk_id: u32, price: u64) -> TransactionData {
        TransactionData {
            tx_type: TransactionType::SellChunk { chunk_id, price },
            from: self.from_address.clone(),
            value: 0,
            gas_price: self.gas_price,
            gas_amount: self.gas_amount,
            data: Vec::new(),
        }
    }

    /// Build a bid transaction
    pub fn build_bid(&self, chunk_id: u32, amount: u64) -> TransactionData {
        TransactionData {
            tx_type: TransactionType::PlaceBid { chunk_id, amount },
            from: self.from_address.clone(),
            value: amount,
            gas_price: self.gas_price,
            gas_amount: self.gas_amount,
            data: Vec::new(),
        }
    }

    /// Build a pixel submit transaction
    pub fn build_submit_pixels(&self, chunk_id: u32, pixels: Vec<u8>) -> TransactionData {
        let pixel_count = pixels.len() as u32;
        TransactionData {
            tx_type: TransactionType::SubmitPixels { chunk_id, pixel_count },
            from: self.from_address.clone(),
            value: 0,
            gas_price: self.gas_price,
            gas_amount: self.gas_amount,
            data: pixels,
        }
    }

    /// Build a reward claim transaction
    pub fn build_claim_reward(&self, chunk_id: u32) -> TransactionData {
        TransactionData {
            tx_type: TransactionType::ClaimReward { chunk_id },
            from: self.from_address.clone(),
            value: 0,
            gas_price: self.gas_price,
            gas_amount: self.gas_amount,
            data: Vec::new(),
        }
    }
}

/// Raw transaction data ready for submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub tx_type: TransactionType,
    pub from: String,
    pub value: u64,
    pub gas_price: u64,
    pub gas_amount: u64,
    pub data: Vec<u8>,
}

/// Batch transaction handler for large data submissions
/// Used when pixel data exceeds single transaction limit (32KB per block)
pub struct BatchTransactionHandler {
    max_batch_size: usize,
}

impl BatchTransactionHandler {
    pub fn new() -> Self {
        // Max 32KB per block as per Alephium limits
        Self {
            max_batch_size: 32 * 1024,
        }
    }

    /// Split large pixel data into batches
    pub fn create_pixel_batches(
        &self,
        chunk_id: u32,
        pixels: Vec<u8>,
    ) -> Vec<Vec<u8>> {
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();

        for pixel in pixels {
            if current_batch.len() >= self.max_batch_size {
                batches.push(current_batch);
                current_batch = Vec::new();
            }
            current_batch.push(pixel);
        }

        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        batches
    }

    /// Check if data needs batching
    pub fn needs_batching(&self, data_size: usize) -> bool {
        data_size > self.max_batch_size
    }
}

impl Default for BatchTransactionHandler {
    fn default() -> Self {
        Self::new()
    }
}
