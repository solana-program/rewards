use pinocchio::error::ProgramError;

use crate::{errors::RewardsProgramError, require_len, traits::InstructionData};

pub struct CreateMerkleDistributionData {
    pub bump: u8,
    pub amount: u64,
    pub merkle_root: [u8; 32],
    pub total_amount: u64,
    pub clawback_ts: i64,
}

impl<'a> TryFrom<&'a [u8]> for CreateMerkleDistributionData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let amount = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let merkle_root: [u8; 32] = data[9..41].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let total_amount =
            u64::from_le_bytes(data[41..49].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let clawback_ts =
            i64::from_le_bytes(data[49..57].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, amount, merkle_root, total_amount, clawback_ts })
    }
}

impl<'a> InstructionData<'a> for CreateMerkleDistributionData {
    const LEN: usize = 1 + 8 + 32 + 8 + 8; // bump + amount + merkle_root + total_amount + clawback_ts = 57

    fn validate(&self) -> Result<(), ProgramError> {
        if self.amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        if self.total_amount == 0 {
            return Err(RewardsProgramError::InvalidAmount.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_data() -> [u8; 57] {
        let mut data = [0u8; 57];
        data[0] = 255; // bump
        data[1..9].copy_from_slice(&1000u64.to_le_bytes()); // amount
        data[9..41].copy_from_slice(&[1u8; 32]); // merkle_root
        data[41..49].copy_from_slice(&5000u64.to_le_bytes()); // total_amount
        data[49..57].copy_from_slice(&1700000000i64.to_le_bytes()); // clawback_ts
        data
    }

    #[test]
    fn test_try_from_valid_data() {
        let data = create_valid_data();
        let result = CreateMerkleDistributionData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.bump, 255);
        assert_eq!(parsed.amount, 1000);
        assert_eq!(parsed.merkle_root, [1u8; 32]);
        assert_eq!(parsed.total_amount, 5000);
        assert_eq!(parsed.clawback_ts, 1700000000);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 50];
        let result = CreateMerkleDistributionData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = create_valid_data();
        let parsed = CreateMerkleDistributionData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_amount() {
        let mut data = create_valid_data();
        data[1..9].copy_from_slice(&0u64.to_le_bytes());
        let parsed = CreateMerkleDistributionData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidAmount.into()));
    }

    #[test]
    fn test_validate_zero_total_amount() {
        let mut data = create_valid_data();
        data[41..49].copy_from_slice(&0u64.to_le_bytes());
        let parsed = CreateMerkleDistributionData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.validate().err(), Some(RewardsProgramError::InvalidAmount.into()));
    }
}
