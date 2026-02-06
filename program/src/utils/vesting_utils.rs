use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, CodamaType)]
pub enum VestingSchedule {
    Immediate {},
    Linear { start_ts: i64, end_ts: i64 },
    Cliff { cliff_ts: i64 },
    CliffLinear { start_ts: i64, cliff_ts: i64, end_ts: i64 },
}

impl VestingSchedule {
    pub fn validate(&self) -> Result<(), ProgramError> {
        match self {
            VestingSchedule::Immediate {} => Ok(()),
            VestingSchedule::Linear { start_ts, end_ts } => {
                if *end_ts <= *start_ts {
                    return Err(RewardsProgramError::InvalidTimeWindow.into());
                }
                Ok(())
            }
            VestingSchedule::Cliff { cliff_ts } => {
                if *cliff_ts <= 0 {
                    return Err(RewardsProgramError::InvalidCliffTimestamp.into());
                }
                Ok(())
            }
            VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts } => {
                if *end_ts <= *start_ts {
                    return Err(RewardsProgramError::InvalidTimeWindow.into());
                }
                if *cliff_ts < *start_ts || *cliff_ts > *end_ts {
                    return Err(RewardsProgramError::InvalidCliffTimestamp.into());
                }
                Ok(())
            }
        }
    }

    pub fn calculate_unlocked(&self, total_amount: u64, current_ts: i64) -> Result<u64, ProgramError> {
        match self {
            VestingSchedule::Immediate {} => Ok(total_amount),
            VestingSchedule::Linear { start_ts, end_ts } => {
                calculate_linear_unlock(total_amount, *start_ts, *end_ts, current_ts)
            }
            VestingSchedule::Cliff { cliff_ts } => {
                if current_ts < *cliff_ts {
                    Ok(0)
                } else {
                    Ok(total_amount)
                }
            }
            VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts } => {
                if current_ts < *cliff_ts {
                    return Ok(0);
                }
                calculate_linear_unlock(total_amount, *start_ts, *end_ts, current_ts)
            }
        }
    }

    pub fn byte_len(&self) -> usize {
        match self {
            VestingSchedule::Immediate {} => 1,
            VestingSchedule::Linear { .. } => 17,
            VestingSchedule::Cliff { .. } => 9,
            VestingSchedule::CliffLinear { .. } => 25,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            VestingSchedule::Immediate {} => {
                alloc::vec![0]
            }
            VestingSchedule::Linear { start_ts, end_ts } => {
                let mut data = Vec::with_capacity(17);
                data.push(1);
                data.extend_from_slice(&start_ts.to_le_bytes());
                data.extend_from_slice(&end_ts.to_le_bytes());
                data
            }
            VestingSchedule::Cliff { cliff_ts } => {
                let mut data = Vec::with_capacity(9);
                data.push(2);
                data.extend_from_slice(&cliff_ts.to_le_bytes());
                data
            }
            VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts } => {
                let mut data = Vec::with_capacity(25);
                data.push(3);
                data.extend_from_slice(&start_ts.to_le_bytes());
                data.extend_from_slice(&cliff_ts.to_le_bytes());
                data.extend_from_slice(&end_ts.to_le_bytes());
                data
            }
        }
    }

    /// Write enum bytes into a buffer without allocation. Returns bytes written.
    pub fn write_bytes(&self, buf: &mut [u8]) -> usize {
        match self {
            VestingSchedule::Immediate {} => {
                buf[0] = 0;
                1
            }
            VestingSchedule::Linear { start_ts, end_ts } => {
                buf[0] = 1;
                buf[1..9].copy_from_slice(&start_ts.to_le_bytes());
                buf[9..17].copy_from_slice(&end_ts.to_le_bytes());
                17
            }
            VestingSchedule::Cliff { cliff_ts } => {
                buf[0] = 2;
                buf[1..9].copy_from_slice(&cliff_ts.to_le_bytes());
                9
            }
            VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts } => {
                buf[0] = 3;
                buf[1..9].copy_from_slice(&start_ts.to_le_bytes());
                buf[9..17].copy_from_slice(&cliff_ts.to_le_bytes());
                buf[17..25].copy_from_slice(&end_ts.to_le_bytes());
                25
            }
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), ProgramError> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        match data[0] {
            0 => Ok((VestingSchedule::Immediate {}, 1)),
            1 => {
                if data.len() < 17 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let start_ts =
                    i64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                let end_ts =
                    i64::from_le_bytes(data[9..17].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                Ok((VestingSchedule::Linear { start_ts, end_ts }, 17))
            }
            2 => {
                if data.len() < 9 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let cliff_ts =
                    i64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                Ok((VestingSchedule::Cliff { cliff_ts }, 9))
            }
            3 => {
                if data.len() < 25 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let start_ts =
                    i64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                let cliff_ts =
                    i64::from_le_bytes(data[9..17].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                let end_ts =
                    i64::from_le_bytes(data[17..25].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                Ok((VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts }, 25))
            }
            _ => Err(RewardsProgramError::InvalidScheduleType.into()),
        }
    }
}

fn calculate_linear_unlock(
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

    // --- Linear unlock helper tests ---

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
    fn test_linear_unlock_at_start() {
        let result = calculate_linear_unlock(1000, 100, 200, 100).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_linear_unlock_at_end() {
        let result = calculate_linear_unlock(1000, 100, 200, 200).unwrap();
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_linear_unlock_three_quarters() {
        let result = calculate_linear_unlock(1000, 0, 100, 75).unwrap();
        assert_eq!(result, 750);
    }

    #[test]
    fn test_linear_unlock_large_amounts() {
        let result = calculate_linear_unlock(u64::MAX, 0, 100, 50).unwrap();
        assert_eq!(result, u64::MAX / 2);
    }

    // --- validate ---

    #[test]
    fn test_validate_immediate() {
        assert!(VestingSchedule::Immediate {}.validate().is_ok());
    }

    #[test]
    fn test_validate_linear_valid() {
        assert!(VestingSchedule::Linear { start_ts: 100, end_ts: 200 }.validate().is_ok());
    }

    #[test]
    fn test_validate_linear_equal_times() {
        assert!(VestingSchedule::Linear { start_ts: 100, end_ts: 100 }.validate().is_err());
    }

    #[test]
    fn test_validate_linear_reversed() {
        assert!(VestingSchedule::Linear { start_ts: 200, end_ts: 100 }.validate().is_err());
    }

    #[test]
    fn test_validate_cliff_valid() {
        assert!(VestingSchedule::Cliff { cliff_ts: 100 }.validate().is_ok());
    }

    #[test]
    fn test_validate_cliff_zero() {
        assert!(VestingSchedule::Cliff { cliff_ts: 0 }.validate().is_err());
    }

    #[test]
    fn test_validate_cliff_linear_valid() {
        assert!(VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 }.validate().is_ok());
    }

    #[test]
    fn test_validate_cliff_linear_cliff_at_start() {
        assert!(VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 100, end_ts: 400 }.validate().is_ok());
    }

    #[test]
    fn test_validate_cliff_linear_cliff_at_end() {
        assert!(VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 400, end_ts: 400 }.validate().is_ok());
    }

    #[test]
    fn test_validate_cliff_linear_cliff_before_start() {
        assert!(VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 50, end_ts: 400 }.validate().is_err());
    }

    #[test]
    fn test_validate_cliff_linear_cliff_after_end() {
        assert!(VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 500, end_ts: 400 }.validate().is_err());
    }

    #[test]
    fn test_validate_cliff_linear_reversed_times() {
        assert!(VestingSchedule::CliffLinear { start_ts: 400, cliff_ts: 200, end_ts: 100 }.validate().is_err());
    }

    // --- calculate_unlocked: Immediate ---

    #[test]
    fn test_unlocked_immediate() {
        let s = VestingSchedule::Immediate {};
        assert_eq!(s.calculate_unlocked(1000, 0).unwrap(), 1000);
        assert_eq!(s.calculate_unlocked(1000, 999999).unwrap(), 1000);
    }

    // --- calculate_unlocked: Linear ---

    #[test]
    fn test_unlocked_linear_before_start() {
        let s = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        assert_eq!(s.calculate_unlocked(1000, 50).unwrap(), 0);
    }

    #[test]
    fn test_unlocked_linear_at_start() {
        let s = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        assert_eq!(s.calculate_unlocked(1000, 100).unwrap(), 0);
    }

    #[test]
    fn test_unlocked_linear_midpoint() {
        let s = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        assert_eq!(s.calculate_unlocked(1000, 150).unwrap(), 500);
    }

    #[test]
    fn test_unlocked_linear_at_end() {
        let s = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        assert_eq!(s.calculate_unlocked(1000, 200).unwrap(), 1000);
    }

    #[test]
    fn test_unlocked_linear_after_end() {
        let s = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        assert_eq!(s.calculate_unlocked(1000, 300).unwrap(), 1000);
    }

    // --- calculate_unlocked: Cliff ---

    #[test]
    fn test_unlocked_cliff_before() {
        let s = VestingSchedule::Cliff { cliff_ts: 100 };
        assert_eq!(s.calculate_unlocked(1000, 50).unwrap(), 0);
        assert_eq!(s.calculate_unlocked(1000, 99).unwrap(), 0);
    }

    #[test]
    fn test_unlocked_cliff_at() {
        let s = VestingSchedule::Cliff { cliff_ts: 100 };
        assert_eq!(s.calculate_unlocked(1000, 100).unwrap(), 1000);
    }

    #[test]
    fn test_unlocked_cliff_after() {
        let s = VestingSchedule::Cliff { cliff_ts: 100 };
        assert_eq!(s.calculate_unlocked(1000, 200).unwrap(), 1000);
    }

    // --- calculate_unlocked: CliffLinear ---

    #[test]
    fn test_unlocked_cliff_linear_before_cliff() {
        // 4-year vest (0..400), cliff at 100 (1 year)
        let s = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        assert_eq!(s.calculate_unlocked(1000, 50).unwrap(), 0);
        assert_eq!(s.calculate_unlocked(1000, 99).unwrap(), 0);
    }

    #[test]
    fn test_unlocked_cliff_linear_at_cliff() {
        // At cliff (100), accumulated linear = 1000 * 100/400 = 250
        let s = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        assert_eq!(s.calculate_unlocked(1000, 100).unwrap(), 250);
    }

    #[test]
    fn test_unlocked_cliff_linear_after_cliff() {
        // At 200, linear = 1000 * 200/400 = 500
        let s = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        assert_eq!(s.calculate_unlocked(1000, 200).unwrap(), 500);
    }

    #[test]
    fn test_unlocked_cliff_linear_at_end() {
        let s = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        assert_eq!(s.calculate_unlocked(1000, 400).unwrap(), 1000);
    }

    #[test]
    fn test_unlocked_cliff_linear_after_end() {
        let s = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        assert_eq!(s.calculate_unlocked(1000, 999).unwrap(), 1000);
    }

    #[test]
    fn test_unlocked_cliff_linear_cliff_at_start() {
        // cliff == start means no cliff delay, behaves like linear
        let s = VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 100, end_ts: 200 };
        assert_eq!(s.calculate_unlocked(1000, 100).unwrap(), 0);
        assert_eq!(s.calculate_unlocked(1000, 150).unwrap(), 500);
        assert_eq!(s.calculate_unlocked(1000, 200).unwrap(), 1000);
    }

    // --- to_bytes / from_bytes roundtrip ---

    #[test]
    fn test_bytes_roundtrip_immediate() {
        let s = VestingSchedule::Immediate {};
        let bytes = s.to_bytes();
        assert_eq!(bytes, [0]);
        let (parsed, consumed) = VestingSchedule::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, s);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_bytes_roundtrip_linear() {
        let s = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        let bytes = s.to_bytes();
        assert_eq!(bytes.len(), 17);
        assert_eq!(bytes[0], 1);
        let (parsed, consumed) = VestingSchedule::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, s);
        assert_eq!(consumed, 17);
    }

    #[test]
    fn test_bytes_roundtrip_cliff() {
        let s = VestingSchedule::Cliff { cliff_ts: 150 };
        let bytes = s.to_bytes();
        assert_eq!(bytes.len(), 9);
        assert_eq!(bytes[0], 2);
        let (parsed, consumed) = VestingSchedule::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, s);
        assert_eq!(consumed, 9);
    }

    #[test]
    fn test_bytes_roundtrip_cliff_linear() {
        let s = VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 200, end_ts: 400 };
        let bytes = s.to_bytes();
        assert_eq!(bytes.len(), 25);
        assert_eq!(bytes[0], 3);
        let (parsed, consumed) = VestingSchedule::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, s);
        assert_eq!(consumed, 25);
    }

    #[test]
    fn test_from_bytes_empty() {
        assert!(VestingSchedule::from_bytes(&[]).is_err());
    }

    #[test]
    fn test_from_bytes_invalid_discriminant() {
        assert!(VestingSchedule::from_bytes(&[4]).is_err());
        assert!(VestingSchedule::from_bytes(&[255]).is_err());
    }

    #[test]
    fn test_from_bytes_truncated_linear() {
        let mut bytes = VestingSchedule::Linear { start_ts: 100, end_ts: 200 }.to_bytes();
        bytes.truncate(10); // cut off end_ts
        assert!(VestingSchedule::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_from_bytes_truncated_cliff_linear() {
        let mut bytes = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 }.to_bytes();
        bytes.truncate(20); // cut off end_ts
        assert!(VestingSchedule::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_write_bytes_matches_to_bytes() {
        let schedules = [
            VestingSchedule::Immediate {},
            VestingSchedule::Linear { start_ts: 100, end_ts: 200 },
            VestingSchedule::Cliff { cliff_ts: 150 },
            VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 200, end_ts: 400 },
        ];
        for s in schedules {
            let vec_bytes = s.to_bytes();
            let mut buf = [0u8; 25];
            let written = s.write_bytes(&mut buf);
            assert_eq!(&buf[..written], &vec_bytes[..]);
        }
    }

    #[test]
    fn test_from_bytes_with_trailing_data() {
        let mut bytes = VestingSchedule::Linear { start_ts: 100, end_ts: 200 }.to_bytes();
        bytes.extend_from_slice(&[0xFF; 10]); // extra trailing data
        let (parsed, consumed) = VestingSchedule::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
        assert_eq!(consumed, 17);
    }
}
