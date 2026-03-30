pub struct TransactionPreview {
    pub gas_estimate: u64,
    pub total_cost: u64,
    pub can_execute: bool,
    pub error_message: Option<String>,
}

pub fn preview_transaction(chunk_price: u64, balance: u64) -> TransactionPreview {
    let gas_estimate = 100_000; // Примерная цена газа
    let total_cost = chunk_price + gas_estimate;

    if balance < total_cost {
        return TransactionPreview {
            gas_estimate,
            total_cost,
            can_execute: false,
            error_message: Some("Insufficient balance".to_string()),
        };
    }

    TransactionPreview {
        gas_estimate,
        total_cost,
        can_execute: true,
        error_message: None,
    }
}