use pinocchio::error::ProgramError;

use crate::utils::VestingSchedule;

/// Interface for types that provide vesting schedule parameters.
///
/// Both DirectRecipient (stores in account) and MerkleClaimData (from instruction)
/// provide vesting params. This trait enables shared unlock calculation logic
/// via the VestingSchedule enum.
pub trait VestingParams {
    /// Total amount subject to vesting
    fn total_amount(&self) -> u64;

    /// The vesting schedule for this allocation
    fn vesting_schedule(&self) -> VestingSchedule;

    /// Calculates the unlocked amount at the given timestamp based on schedule
    #[inline(always)]
    fn calculate_unlocked(&self, current_ts: i64) -> Result<u64, ProgramError> {
        self.vesting_schedule().calculate_unlocked(self.total_amount(), current_ts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockVesting {
        total_amount: u64,
        schedule: VestingSchedule,
    }

    impl VestingParams for MockVesting {
        fn total_amount(&self) -> u64 {
            self.total_amount
        }

        fn vesting_schedule(&self) -> VestingSchedule {
            self.schedule
        }
    }

    #[test]
    fn test_calculate_unlocked_linear_before_start() {
        let vesting =
            MockVesting { total_amount: 1000, schedule: VestingSchedule::Linear { start_ts: 100, end_ts: 200 } };
        assert_eq!(vesting.calculate_unlocked(50).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_linear_at_start() {
        let vesting =
            MockVesting { total_amount: 1000, schedule: VestingSchedule::Linear { start_ts: 100, end_ts: 200 } };
        assert_eq!(vesting.calculate_unlocked(100).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_linear_midpoint() {
        let vesting =
            MockVesting { total_amount: 1000, schedule: VestingSchedule::Linear { start_ts: 100, end_ts: 200 } };
        assert_eq!(vesting.calculate_unlocked(150).unwrap(), 500);
    }

    #[test]
    fn test_calculate_unlocked_linear_at_end() {
        let vesting =
            MockVesting { total_amount: 1000, schedule: VestingSchedule::Linear { start_ts: 100, end_ts: 200 } };
        assert_eq!(vesting.calculate_unlocked(200).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_linear_after_end() {
        let vesting =
            MockVesting { total_amount: 1000, schedule: VestingSchedule::Linear { start_ts: 100, end_ts: 200 } };
        assert_eq!(vesting.calculate_unlocked(300).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_linear_quarter() {
        let vesting =
            MockVesting { total_amount: 1000, schedule: VestingSchedule::Linear { start_ts: 0, end_ts: 100 } };
        assert_eq!(vesting.calculate_unlocked(25).unwrap(), 250);
    }

    #[test]
    fn test_calculate_unlocked_immediate_before_start() {
        let vesting = MockVesting { total_amount: 1000, schedule: VestingSchedule::Immediate {} };
        assert_eq!(vesting.calculate_unlocked(50).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_immediate_at_start() {
        let vesting = MockVesting { total_amount: 1000, schedule: VestingSchedule::Immediate {} };
        assert_eq!(vesting.calculate_unlocked(100).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_immediate_midpoint() {
        let vesting = MockVesting { total_amount: 1000, schedule: VestingSchedule::Immediate {} };
        assert_eq!(vesting.calculate_unlocked(150).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_cliff_before() {
        let vesting = MockVesting { total_amount: 1000, schedule: VestingSchedule::Cliff { cliff_ts: 100 } };
        assert_eq!(vesting.calculate_unlocked(50).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_cliff_at() {
        let vesting = MockVesting { total_amount: 1000, schedule: VestingSchedule::Cliff { cliff_ts: 100 } };
        assert_eq!(vesting.calculate_unlocked(100).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_cliff_linear_before_cliff() {
        let vesting = MockVesting {
            total_amount: 1000,
            schedule: VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 },
        };
        assert_eq!(vesting.calculate_unlocked(50).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_cliff_linear_at_cliff() {
        let vesting = MockVesting {
            total_amount: 1000,
            schedule: VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 },
        };
        assert_eq!(vesting.calculate_unlocked(100).unwrap(), 250);
    }

    #[test]
    fn test_calculate_unlocked_cliff_linear_at_end() {
        let vesting = MockVesting {
            total_amount: 1000,
            schedule: VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 },
        };
        assert_eq!(vesting.calculate_unlocked(400).unwrap(), 1000);
    }
}
