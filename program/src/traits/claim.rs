use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;

/// Common interface for claim tracking accounts.
///
/// Both MerkleClaim and DirectRecipient track how much a user has claimed.
/// This trait abstracts the common operations.
pub trait ClaimTracker {
    /// Returns the amount already claimed by this recipient
    fn claimed_amount(&self) -> u64;

    /// Sets the claimed amount
    fn set_claimed_amount(&mut self, amount: u64) -> Result<(), ProgramError>;

    /// Calculates the claimable amount given the total unlocked amount
    #[inline(always)]
    fn claimable_amount(&self, unlocked: u64) -> Result<u64, RewardsProgramError> {
        unlocked.checked_sub(self.claimed_amount()).ok_or(RewardsProgramError::MathOverflow)
    }

    /// Adds to the claimed amount with overflow checking
    #[inline(always)]
    fn add_claimed(&mut self, amount: u64) -> Result<(), ProgramError> {
        let new_amount = self.claimed_amount().checked_add(amount).ok_or(RewardsProgramError::MathOverflow)?;
        self.set_claimed_amount(new_amount)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockClaim {
        claimed_amount: u64,
    }

    impl ClaimTracker for MockClaim {
        fn claimed_amount(&self) -> u64 {
            self.claimed_amount
        }

        fn set_claimed_amount(&mut self, amount: u64) -> Result<(), ProgramError> {
            if amount < self.claimed_amount {
                return Err(RewardsProgramError::ClaimedAmountDecreased.into());
            }
            self.claimed_amount = amount;
            Ok(())
        }
    }

    #[test]
    fn test_claimable_amount_fresh() {
        let claim = MockClaim { claimed_amount: 0 };
        assert_eq!(claim.claimable_amount(1000).unwrap(), 1000);
    }

    #[test]
    fn test_claimable_amount_partial() {
        let claim = MockClaim { claimed_amount: 300 };
        assert_eq!(claim.claimable_amount(1000).unwrap(), 700);
    }

    #[test]
    fn test_claimable_amount_overflow() {
        let claim = MockClaim { claimed_amount: 1500 };
        assert!(claim.claimable_amount(1000).is_err());
    }

    #[test]
    fn test_add_claimed_success() {
        let mut claim = MockClaim { claimed_amount: 0 };
        assert!(claim.add_claimed(500).is_ok());
        assert_eq!(claim.claimed_amount(), 500);
    }

    #[test]
    fn test_add_claimed_accumulates() {
        let mut claim = MockClaim { claimed_amount: 200 };
        claim.add_claimed(300).unwrap();
        assert_eq!(claim.claimed_amount(), 500);
    }

    #[test]
    fn test_add_claimed_overflow() {
        let mut claim = MockClaim { claimed_amount: u64::MAX };
        assert!(claim.add_claimed(1).is_err());
    }

    #[test]
    fn test_set_claimed_amount_rejects_decrease() {
        let mut claim = MockClaim { claimed_amount: 500 };
        assert!(claim.set_claimed_amount(400).is_err());
        assert_eq!(claim.claimed_amount(), 500);
    }
}
