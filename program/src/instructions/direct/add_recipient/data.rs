use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData, utils::VestingSchedule};

/// Instruction data for AddDirectRecipient.
///
/// Variable-length due to the VestingSchedule enum.
pub struct AddDirectRecipientData {
    /// Bump for the recipient PDA
    pub bump: u8,
    /// Token amount allocated to this recipient
    pub amount: u64,
    /// Vesting schedule for this recipient's allocation
    pub schedule: VestingSchedule,
}

impl<'a> TryFrom<&'a [u8]> for AddDirectRecipientData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let amount = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let (schedule, _) = VestingSchedule::from_bytes(&data[9..])?;

        Ok(Self { bump, amount, schedule })
    }
}

impl<'a> InstructionData<'a> for AddDirectRecipientData {
    const LEN: usize = 1 + 8 + 1; // bump + amount + min schedule (Immediate)

    fn validate(&self) -> Result<(), ProgramError> {
        if self.amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        self.schedule.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    fn create_linear_data() -> Vec<u8> {
        let schedule = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        let schedule_bytes = schedule.to_bytes();
        let mut data = Vec::with_capacity(9 + schedule_bytes.len());
        data.push(255); // bump
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        data.extend_from_slice(&schedule_bytes);
        data
    }

    #[test]
    fn test_try_from_valid_linear() {
        let data = create_linear_data();
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.bump, 255);
        assert_eq!(parsed.amount, 1000);
        assert_eq!(parsed.schedule, VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
    }

    #[test]
    fn test_try_from_valid_immediate() {
        let mut data = Vec::new();
        data.push(1); // bump
        data.extend_from_slice(&500u64.to_le_bytes()); // amount
        data.extend_from_slice(&VestingSchedule::Immediate {}.to_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.bump, 1);
        assert_eq!(parsed.amount, 500);
        assert_eq!(parsed.schedule, VestingSchedule::Immediate {});
    }

    #[test]
    fn test_try_from_valid_cliff_linear() {
        let schedule = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        let mut data = Vec::new();
        data.push(200); // bump
        data.extend_from_slice(&5000u64.to_le_bytes()); // amount
        data.extend_from_slice(&schedule.to_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.schedule, schedule);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 5];
        let result = AddDirectRecipientData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_try_from_invalid_schedule_discriminant() {
        let mut data = Vec::new();
        data.push(1); // bump
        data.extend_from_slice(&1000u64.to_le_bytes()); // amount
        data.push(255); // invalid schedule discriminant
        let result = AddDirectRecipientData::try_from(&data[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_success() {
        let data = create_linear_data();
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_amount() {
        let mut data = Vec::new();
        data.push(1); // bump
        data.extend_from_slice(&0u64.to_le_bytes()); // amount = 0
        data.extend_from_slice(&VestingSchedule::Immediate {}.to_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidAmount.into()));
    }

    #[test]
    fn test_validate_invalid_time_window() {
        let schedule = VestingSchedule::Linear { start_ts: 200, end_ts: 100 };
        let mut data = Vec::new();
        data.push(1); // bump
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&schedule.to_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidTimeWindow.into()));
    }

    #[test]
    fn test_validate_cliff_linear() {
        let schedule = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        let mut data = Vec::new();
        data.push(1);
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&schedule.to_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_validate_cliff_linear_invalid_cliff() {
        let schedule = VestingSchedule::CliffLinear { start_ts: 100, cliff_ts: 50, end_ts: 400 };
        let mut data = Vec::new();
        data.push(1);
        data.extend_from_slice(&1000u64.to_le_bytes());
        data.extend_from_slice(&schedule.to_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_err());
    }
}
