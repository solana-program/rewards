use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct CreateDirectDistributionData {
    pub bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for CreateDirectDistributionData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];

        Ok(Self { bump })
    }
}

impl<'a> InstructionData<'a> for CreateDirectDistributionData {
    const LEN: usize = 1; // bump

    fn validate(&self) -> Result<(), ProgramError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_valid_data() {
        let data = [255u8]; // bump
        let result = CreateDirectDistributionData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.bump, 255);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data: [u8; 0] = [];
        let result = CreateDirectDistributionData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = [255u8];
        let parsed = CreateDirectDistributionData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }
}
