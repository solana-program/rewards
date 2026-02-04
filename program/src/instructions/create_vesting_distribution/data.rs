use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

pub struct CreateVestingDistributionData {
    pub bump: u8,
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for CreateVestingDistributionData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let amount = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, amount })
    }
}

impl<'a> InstructionData<'a> for CreateVestingDistributionData {
    const LEN: usize = 1 + 8; // bump + amount

    fn validate(&self) -> Result<(), ProgramError> {
        if self.amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        Ok(())
    }
}
