use codama::CodamaErrors;
use pinocchio::error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Rewards Program.
#[derive(Clone, Debug, Eq, PartialEq, Error, CodamaErrors)]
pub enum RewardsProgramError {
    /// (0) Invalid amount specified
    #[error("Invalid amount specified")]
    InvalidAmount,

    /// (1) Invalid time window configuration
    #[error("Invalid time window configuration")]
    InvalidTimeWindow,

    /// (2) Invalid schedule type
    #[error("Invalid schedule type")]
    InvalidScheduleType,

    /// (3) Unauthorized authority
    #[error("Unauthorized authority")]
    UnauthorizedAuthority,

    /// (4) Unauthorized recipient
    #[error("Unauthorized recipient")]
    UnauthorizedRecipient,

    /// (5) Insufficient funds in distribution
    #[error("Insufficient funds in distribution")]
    InsufficientFunds,

    /// (6) Nothing available to claim
    #[error("Nothing available to claim")]
    NothingToClaim,

    /// (7) Math overflow occurred
    #[error("Math overflow occurred")]
    MathOverflow,

    /// (8) Invalid account data
    #[error("Invalid account data")]
    InvalidAccountData,

    /// (9) Event authority PDA is invalid
    #[error("Event authority PDA is invalid")]
    InvalidEventAuthority,

    /// (10) Mint has PermanentDelegate extension which is not allowed
    #[error("Mint has PermanentDelegate extension which is not allowed")]
    PermanentDelegateNotAllowed,

    /// (11) Mint has NonTransferable extension which is not allowed
    #[error("Mint has NonTransferable extension which is not allowed")]
    NonTransferableNotAllowed,

    /// (12) Mint has Pausable extension which is not allowed
    #[error("Mint has Pausable extension which is not allowed")]
    PausableNotAllowed,

    /// (13) Rent calculation failed
    #[error("Rent calculation failed")]
    RentCalculationFailed,

    /// (14) Requested claim amount exceeds available balance
    #[error("Requested claim amount exceeds available balance")]
    ExceedsClaimableAmount,

    /// (15) Invalid merkle proof
    #[error("Invalid merkle proof")]
    InvalidMerkleProof,

    /// (16) Clawback timestamp not yet reached
    #[error("Clawback timestamp not yet reached")]
    ClawbackNotReached,

    /// (17) Claim has not been fully vested
    #[error("Claim has not been fully vested")]
    ClaimNotFullyVested,
}

impl From<RewardsProgramError> for ProgramError {
    fn from(e: RewardsProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
