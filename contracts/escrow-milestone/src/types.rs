use soroban_sdk::{contracttype, Address, BytesN};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MilestoneStatus {
    Pending = 0,
    Submitted = 1,
    Approved = 2,
    Released = 3,
    Disputed = 4,
    Cancelled = 5,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Milestone {
    pub id: u32,
    pub description_hash: BytesN<32>,
    pub amount: i128,
    pub status: MilestoneStatus,
    pub proof_hash: Option<BytesN<32>>,
    pub submitted_at: u64,
    pub approved_at: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MilestoneInit {
    pub id: u32,
    pub description_hash: BytesN<32>,
    pub amount: i128,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MilestoneEscrow {
    pub client: Address,
    pub contractor: Address,
    pub arbiter: Option<Address>,
    pub token: Address,
    pub total_amount: i128,
    pub released_amount: i128,
    pub is_funded: bool,
    pub milestone_count: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Milestone(u32),
}
