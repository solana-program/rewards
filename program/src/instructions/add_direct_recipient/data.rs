use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData, utils::VestingScheduleType};

pub struct AddDirectRecipientData {
    pub bump: u8,
    pub amount: u64,
    pub schedule_type: u8,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl<'a> TryFrom<&'a [u8]> for AddDirectRecipientData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let amount = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let schedule_type = data[9];
        let start_ts = i64::from_le_bytes(data[10..18].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let end_ts = i64::from_le_bytes(data[18..26].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, amount, schedule_type, start_ts, end_ts })
    }
}

impl<'a> InstructionData<'a> for AddDirectRecipientData {
    const LEN: usize = 1 + 8 + 1 + 8 + 8; // bump + amount + schedule_type + start_ts + end_ts

    fn validate(&self) -> Result<(), ProgramError> {
        if self.amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        VestingScheduleType::from_u8(self.schedule_type).ok_or(RewardsProgramError::InvalidScheduleType)?;
        if self.end_ts <= self.start_ts {
            return Err(RewardsProgramError::InvalidTimeWindow.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_data() -> [u8; 26] {
        let mut data = [0u8; 26];
        data[0] = 255; // bump
        data[1..9].copy_from_slice(&1000u64.to_le_bytes()); // amount
        data[9] = VestingScheduleType::Linear as u8; // schedule_type
        data[10..18].copy_from_slice(&100i64.to_le_bytes()); // start_ts
        data[18..26].copy_from_slice(&200i64.to_le_bytes()); // end_ts
        data
    }

    #[test]
    fn test_try_from_valid_data() {
        let data = create_valid_data();
        let result = AddDirectRecipientData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.bump, 255);
        assert_eq!(parsed.amount, 1000);
        assert_eq!(parsed.schedule_type, VestingScheduleType::Linear as u8);
        assert_eq!(parsed.start_ts, 100);
        assert_eq!(parsed.end_ts, 200);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 10];
        let result = AddDirectRecipientData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = create_valid_data();
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_amount() {
        let mut data = create_valid_data();
        data[1..9].copy_from_slice(&0u64.to_le_bytes());
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidAmount.into()));
    }

    #[test]
    fn test_validate_invalid_schedule_type() {
        let mut data = create_valid_data();
        data[9] = 255; // invalid schedule type
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidScheduleType.into()));
    }

    #[test]
    fn test_validate_invalid_time_window_equal() {
        let mut data = create_valid_data();
        data[10..18].copy_from_slice(&100i64.to_le_bytes()); // start_ts
        data[18..26].copy_from_slice(&100i64.to_le_bytes()); // end_ts == start_ts
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidTimeWindow.into()));
    }

    #[test]
    fn test_validate_invalid_time_window_end_before_start() {
        let mut data = create_valid_data();
        data[10..18].copy_from_slice(&200i64.to_le_bytes()); // start_ts
        data[18..26].copy_from_slice(&100i64.to_le_bytes()); // end_ts < start_ts
        let parsed = AddDirectRecipientData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidTimeWindow.into()));
    }
}
