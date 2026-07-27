use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VerificationError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractPaused = 3,
    Unauthorized = 4,
    ValidatorNotFound = 5,
    ValidatorInactive = 6,
    ValidatorAlreadyRegistered = 7,
    PlayerNotFound = 8,
    InvalidInput = 9,
    ReasonTooLong = 10,
    AlreadyConfigured = 11,
    ProgressCallFailed = 12,
    Overflow = 13,
    MilestoneNotFound = 14,
    ValidatorCapReached = 15,
    /// Level-2/3 advance blocked: the player's approving validators do not
    /// span the required minimum number of distinct geographic regions.
    InsufficientRegionDiversity = 16,
    /// `register_validator` was called without a region field on a contract
    /// that has a non-zero MinRegionQuorum set.
    MissingValidatorRegion = 17,
    MilestoneLimitExceeded = 18,
}
