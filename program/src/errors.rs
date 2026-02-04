use codama::CodamaErrors;
use pinocchio::error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Rewards Program.
#[derive(Clone, Debug, Eq, PartialEq, Error, CodamaErrors)]
pub enum RewardsProgramError {
    /// (0) Claim window is not currently active
    #[error("Claim window is not currently active")]
    ClaimWindowNotActive,

    /// (1) Recipient has already claimed this distribution
    #[error("Recipient has already claimed this distribution")]
    AlreadyClaimed,

    /// (2) Invalid amount specified
    #[error("Invalid amount specified")]
    InvalidAmount,

    /// (3) Invalid time window configuration
    #[error("Invalid time window configuration")]
    InvalidTimeWindow,

    /// (4) Invalid schedule type
    #[error("Invalid schedule type")]
    InvalidScheduleType,

    /// (5) Unauthorized authority
    #[error("Unauthorized authority")]
    UnauthorizedAuthority,

    /// (6) Unauthorized recipient
    #[error("Unauthorized recipient")]
    UnauthorizedRecipient,

    /// (7) Insufficient funds in distribution
    #[error("Insufficient funds in distribution")]
    InsufficientFunds,

    /// (8) Nothing available to claim
    #[error("Nothing available to claim")]
    NothingToClaim,

    /// (9) Math overflow occurred
    #[error("Math overflow occurred")]
    MathOverflow,

    /// (10) Invalid account data
    #[error("Invalid account data")]
    InvalidAccountData,

    /// (11) Event authority PDA is invalid
    #[error("Event authority PDA is invalid")]
    InvalidEventAuthority,

    /// (12) Mint has PermanentDelegate extension which is not allowed
    #[error("Mint has PermanentDelegate extension which is not allowed")]
    PermanentDelegateNotAllowed,

    /// (13) Mint has NonTransferable extension which is not allowed
    #[error("Mint has NonTransferable extension which is not allowed")]
    NonTransferableNotAllowed,

    /// (14) Mint has Pausable extension which is not allowed
    #[error("Mint has Pausable extension which is not allowed")]
    PausableNotAllowed,

    /// (15) Rent calculation failed
    #[error("Rent calculation failed")]
    RentCalculationFailed,
}

impl From<RewardsProgramError> for ProgramError {
    fn from(e: RewardsProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let error: ProgramError = RewardsProgramError::ClaimWindowNotActive.into();
        assert_eq!(error, ProgramError::Custom(0));

        let error: ProgramError = RewardsProgramError::AlreadyClaimed.into();
        assert_eq!(error, ProgramError::Custom(1));

        let error: ProgramError = RewardsProgramError::InvalidAmount.into();
        assert_eq!(error, ProgramError::Custom(2));

        let error: ProgramError = RewardsProgramError::InvalidTimeWindow.into();
        assert_eq!(error, ProgramError::Custom(3));

        let error: ProgramError = RewardsProgramError::InvalidScheduleType.into();
        assert_eq!(error, ProgramError::Custom(4));

        let error: ProgramError = RewardsProgramError::UnauthorizedAuthority.into();
        assert_eq!(error, ProgramError::Custom(5));
    }
}
