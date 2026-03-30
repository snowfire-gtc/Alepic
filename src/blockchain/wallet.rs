use crate::blockchain::client::AlephiumClient;
use crate::blockchain::contract::AlepicContract;

pub struct WalletManager {
    pub connected: bool,
    pub address: Option<String>,
    pub balance: u64,
    pub locked_balance: u64,
    client: Option<AlephiumClient>,
    contract: Option<AlepicContract>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            connected: false,
            address: None,
            balance: 0,
            locked_balance: 0,
            client: None,
            contract: None,
        }
    }

    /// Initialize with blockchain connection
    pub fn init(&mut self, node_url: String, contract_address: String) {
        self.client = Some(AlephiumClient::new(node_url));
        self.contract = Some(AlepicContract::new(contract_address));
    }

    pub fn connect(&mut self, address: String) {
        self.address = Some(address.clone());
        self.connected = true;
        // Balance will be fetched from blockchain
        self.balance = 0;
    }

    pub fn disconnect(&mut self) {
        self.address = None;
        self.connected = false;
        self.balance = 0;
        self.locked_balance = 0;
    }

    /// Fetch balance from blockchain
    pub async fn refresh_balance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(client), Some(address)) = (&self.client, &self.address) {
            let account_info = client.get_balance(address).await?;
            self.balance = account_info.balance;
            self.locked_balance = account_info.locked_balance;
        }
        Ok(())
    }

    /// Check if user owns a specific chunk
    pub fn owns_chunk(&self, chunk_owner: &Option<String>) -> bool {
        match (&self.address, chunk_owner) {
            (Some(user_addr), Some(owner)) => user_addr == owner,
            _ => false,
        }
    }

    /// Get contract reference
    pub fn get_contract(&self) -> Option<&AlepicContract> {
        self.contract.as_ref()
    }

    /// Get client reference
    pub fn get_client(&self) -> Option<&AlephiumClient> {
        self.client.as_ref()
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}