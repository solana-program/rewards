use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

/// Instruction data for ClaimVesting
///
/// - `amount`: The amount to claim. If 0, claims all available.
pub struct ClaimVestingData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for ClaimVestingData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let amount = u64::from_le_bytes(data[..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        Ok(Self { amount })
    }
}

impl<'a> InstructionData<'a> for ClaimVestingData {
    const LEN: usize = 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_valid_data() {
        let amount: u64 = 1000;
        let data = amount.to_le_bytes();
        let result = ClaimVestingData::try_from(&data[..]).unwrap();
        assert_eq!(result.amount, 1000);
    }

    #[test]
    fn test_try_from_zero_amount() {
        let amount: u64 = 0;
        let data = amount.to_le_bytes();
        let result = ClaimVestingData::try_from(&data[..]).unwrap();
        assert_eq!(result.amount, 0);
    }

    #[test]
    fn test_try_from_extra_data() {
        let mut data = 500u64.to_le_bytes().to_vec();
        data.extend_from_slice(&[1, 2, 3]);
        let result = ClaimVestingData::try_from(&data[..]).unwrap();
        assert_eq!(result.amount, 500);
    }

    #[test]
    fn test_try_from_insufficient_data() {
        let data = [1, 2, 3];
        let result = ClaimVestingData::try_from(&data[..]);
        assert!(result.is_err());
    }
}
