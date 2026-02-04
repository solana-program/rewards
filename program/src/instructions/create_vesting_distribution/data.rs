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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_data() -> [u8; 9] {
        let mut data = [0u8; 9];
        data[0] = 255; // bump
        data[1..9].copy_from_slice(&1000u64.to_le_bytes()); // amount
        data
    }

    #[test]
    fn test_try_from_valid_data() {
        let data = create_valid_data();
        let result = CreateVestingDistributionData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.bump, 255);
        assert_eq!(parsed.amount, 1000);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 5];
        let result = CreateVestingDistributionData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = create_valid_data();
        let parsed = CreateVestingDistributionData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_amount() {
        let mut data = create_valid_data();
        data[1..9].copy_from_slice(&0u64.to_le_bytes());
        let parsed = CreateVestingDistributionData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidAmount.into()));
    }
}
