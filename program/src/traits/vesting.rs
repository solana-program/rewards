use pinocchio::error::ProgramError;

use crate::utils::{calculate_linear_unlock, VestingScheduleType};

/// Interface for types that provide vesting schedule parameters.
///
/// Both DirectRecipient (stores in account) and MerkleClaimData (from instruction)
/// provide vesting params. This trait enables shared unlock calculation logic.
pub trait VestingParams {
    /// Total amount subject to vesting
    fn total_amount(&self) -> u64;

    /// Vesting start timestamp (unix seconds)
    fn start_ts(&self) -> i64;

    /// Vesting end timestamp (unix seconds)
    fn end_ts(&self) -> i64;

    /// Vesting schedule type
    fn schedule_type(&self) -> VestingScheduleType;

    /// Calculates the unlocked amount at the given timestamp based on schedule type
    #[inline(always)]
    fn calculate_unlocked(&self, current_ts: i64) -> Result<u64, ProgramError> {
        match self.schedule_type() {
            VestingScheduleType::Immediate => Ok(self.total_amount()),
            VestingScheduleType::Linear => {
                calculate_linear_unlock(self.total_amount(), self.start_ts(), self.end_ts(), current_ts)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockVesting {
        total_amount: u64,
        start_ts: i64,
        end_ts: i64,
        schedule_type: VestingScheduleType,
    }

    impl VestingParams for MockVesting {
        fn total_amount(&self) -> u64 {
            self.total_amount
        }

        fn start_ts(&self) -> i64 {
            self.start_ts
        }

        fn end_ts(&self) -> i64 {
            self.end_ts
        }

        fn schedule_type(&self) -> VestingScheduleType {
            self.schedule_type
        }
    }

    #[test]
    fn test_calculate_unlocked_linear_before_start() {
        let vesting =
            MockVesting { total_amount: 1000, start_ts: 100, end_ts: 200, schedule_type: VestingScheduleType::Linear };
        assert_eq!(vesting.calculate_unlocked(50).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_linear_at_start() {
        let vesting =
            MockVesting { total_amount: 1000, start_ts: 100, end_ts: 200, schedule_type: VestingScheduleType::Linear };
        assert_eq!(vesting.calculate_unlocked(100).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_linear_midpoint() {
        let vesting =
            MockVesting { total_amount: 1000, start_ts: 100, end_ts: 200, schedule_type: VestingScheduleType::Linear };
        assert_eq!(vesting.calculate_unlocked(150).unwrap(), 500);
    }

    #[test]
    fn test_calculate_unlocked_linear_at_end() {
        let vesting =
            MockVesting { total_amount: 1000, start_ts: 100, end_ts: 200, schedule_type: VestingScheduleType::Linear };
        assert_eq!(vesting.calculate_unlocked(200).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_linear_after_end() {
        let vesting =
            MockVesting { total_amount: 1000, start_ts: 100, end_ts: 200, schedule_type: VestingScheduleType::Linear };
        assert_eq!(vesting.calculate_unlocked(300).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_linear_quarter() {
        let vesting =
            MockVesting { total_amount: 1000, start_ts: 0, end_ts: 100, schedule_type: VestingScheduleType::Linear };
        assert_eq!(vesting.calculate_unlocked(25).unwrap(), 250);
    }

    #[test]
    fn test_calculate_unlocked_immediate_before_start() {
        let vesting = MockVesting {
            total_amount: 1000,
            start_ts: 100,
            end_ts: 200,
            schedule_type: VestingScheduleType::Immediate,
        };
        assert_eq!(vesting.calculate_unlocked(50).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_immediate_at_start() {
        let vesting = MockVesting {
            total_amount: 1000,
            start_ts: 100,
            end_ts: 200,
            schedule_type: VestingScheduleType::Immediate,
        };
        assert_eq!(vesting.calculate_unlocked(100).unwrap(), 1000);
    }

    #[test]
    fn test_calculate_unlocked_immediate_midpoint() {
        let vesting = MockVesting {
            total_amount: 1000,
            start_ts: 100,
            end_ts: 200,
            schedule_type: VestingScheduleType::Immediate,
        };
        assert_eq!(vesting.calculate_unlocked(150).unwrap(), 1000);
    }
}
