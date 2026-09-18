use soroban_sdk::{contracttype, Address, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MultisigAction {
    Release { to: Address, amount: i128 },
    Refund { to: Address, amount: i128 },
    UpdateThreshold { new_threshold: u32 },
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Proposal {
    pub id: u32,
    pub action: MultisigAction,
    pub approvals_count: u32,
    pub executed: bool,
    pub created_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MultisigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
    pub token: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Proposal(u32),
    Approved(u32, Address),
    ProposalCount,
}
