use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MilestoneError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidMilestoneCount = 3,
    AmountMismatch = 4,
    NotFunded = 5,
    AlreadyFunded = 6,
    MilestoneNotFound = 7,
    InvalidMilestoneStatus = 8,
    Unauthorized = 9,
    ArbiterRequired = 10,
    AllMilestonesProcessed = 11,
}
