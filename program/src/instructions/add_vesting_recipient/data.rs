use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData, utils::VestingScheduleType};

pub struct AddVestingRecipientData {
    pub bump: u8,
    pub amount: u64,
    pub schedule_type: u8,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl<'a> TryFrom<&'a [u8]> for AddVestingRecipientData {
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

impl<'a> InstructionData<'a> for AddVestingRecipientData {
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
