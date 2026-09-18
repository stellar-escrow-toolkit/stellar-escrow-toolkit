use soroban_sdk::{contracttype, Address, BytesN};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EscrowStatus {
    Created = 0,
    Funded = 1,
    Completed = 2,
    Refunded = 3,
    Disputed = 4,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Escrow {
    pub id: u64,
    pub initiator: Address,
    pub beneficiary: Address,
    pub arbiter: Option<Address>,
    pub token: Address,
    pub amount: i128,
    pub funded_amount: i128,
    pub deadline: u64,
    pub status: EscrowStatus,
    pub created_at: u64,
    pub engagement_id: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow,
    Initialized,
}
