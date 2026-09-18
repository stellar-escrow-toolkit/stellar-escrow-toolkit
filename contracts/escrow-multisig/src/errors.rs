use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MultisigError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidThreshold = 3,
    NotSigner = 4,
    ProposalNotFound = 5,
    AlreadyApproved = 6,
    AlreadyExecuted = 7,
    ThresholdNotMet = 8,
    InvalidAmount = 9,
    SignersEmpty = 10,
}
