use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VestingScheduleType {
    Linear = 0,
}

impl VestingScheduleType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(VestingScheduleType::Linear),
            _ => None,
        }
    }
}

pub fn calculate_linear_unlock(
    total_amount: u64,
    start_ts: i64,
    end_ts: i64,
    current_ts: i64,
) -> Result<u64, ProgramError> {
    if current_ts <= start_ts {
        return Ok(0);
    }
    if current_ts >= end_ts {
        return Ok(total_amount);
    }

    let elapsed = current_ts.checked_sub(start_ts).ok_or(RewardsProgramError::MathOverflow)? as u64;
    let duration = end_ts.checked_sub(start_ts).ok_or(RewardsProgramError::MathOverflow)? as u64;

    if duration == 0 {
        return Ok(total_amount);
    }

    let total_128 = total_amount as u128;
    let elapsed_128 = elapsed as u128;
    let duration_128 = duration as u128;

    let result = total_128
        .checked_mul(elapsed_128)
        .ok_or(RewardsProgramError::MathOverflow)?
        .checked_div(duration_128)
        .ok_or(RewardsProgramError::MathOverflow)?;

    Ok(result as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_unlock_before_start() {
        let result = calculate_linear_unlock(1000, 100, 200, 50).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_linear_unlock_after_end() {
        let result = calculate_linear_unlock(1000, 100, 200, 250).unwrap();
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_linear_unlock_midpoint() {
        let result = calculate_linear_unlock(1000, 100, 200, 150).unwrap();
        assert_eq!(result, 500);
    }

    #[test]
    fn test_linear_unlock_quarter() {
        let result = calculate_linear_unlock(1000, 0, 100, 25).unwrap();
        assert_eq!(result, 250);
    }

    #[test]
    fn test_vesting_schedule_type_from_u8() {
        assert_eq!(VestingScheduleType::from_u8(0), Some(VestingScheduleType::Linear));
        assert_eq!(VestingScheduleType::from_u8(1), None);
        assert_eq!(VestingScheduleType::from_u8(255), None);
    }
}
