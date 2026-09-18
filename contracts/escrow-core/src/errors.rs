use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InvalidDeadline = 4,
    InvalidStatus = 5,
    Unauthorized = 6,
    DeadlineNotReached = 7,
    DeadlinePassed = 8,
    EscrowAlreadyFunded = 9,
    EscrowNotFunded = 10,
    ArbiterRequired = 11,
    DisputeResolutionMismatch = 12,
}
