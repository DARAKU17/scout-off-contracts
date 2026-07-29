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
    Overflow = 13,
    DisputeAlreadyExists = 14,
    DisputeNotFound = 15,
    DisputeAlreadyResolved = 16,
    DisputeRequiresJury = 17,
    DisputeDoesNotRequireJury = 18,
    ConflictedValidator = 19,
    VoteAlreadyCast = 20,
    VotingWindowClosed = 21,
    TallyNotReady = 22,
    InvalidJuryConfig = 23,
}
