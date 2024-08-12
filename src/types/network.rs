#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Network {
    ETH,
    BSC,
}

impl Network {
    pub fn chain_id(&self) -> u64 {
        match self {
            Network::ETH => 1,
            Network::BSC => 56,
        }
    }
}
