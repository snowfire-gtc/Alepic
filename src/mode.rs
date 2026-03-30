/// Operation Mode: Real blockchain or Simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Real mode: Connects to Alephium blockchain via bridge
    Real,
    /// Simulation mode: Local storage, fake transactions, no blockchain
    Simulation,
}

impl Default for OperationMode {
    fn default() -> Self {
        OperationMode::Simulation
    }
}

impl OperationMode {
    pub fn is_real(&self) -> bool {
        *self == OperationMode::Real
    }

    pub fn is_simulation(&self) -> bool {
        *self == OperationMode::Simulation
    }
}
