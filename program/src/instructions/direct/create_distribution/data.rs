use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct CreateDirectDistributionData {
    pub bump: u8,
    pub revocable: u8,
    pub clawback_ts: i64,
}

impl<'a> TryFrom<&'a [u8]> for CreateDirectDistributionData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let revocable = data[1];
        let clawback_ts = i64::from_le_bytes(data[2..10].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, revocable, clawback_ts })
    }
}

impl<'a> InstructionData<'a> for CreateDirectDistributionData {
    const LEN: usize = 10; // bump(1) + revocable(1) + clawback_ts(8)

    fn validate(&self) -> Result<(), ProgramError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(bump: u8, revocable: u8, clawback_ts: i64) -> [u8; 10] {
        let mut data = [0u8; 10];
        data[0] = bump;
        data[1] = revocable;
        data[2..10].copy_from_slice(&clawback_ts.to_le_bytes());
        data
    }

    #[test]
    fn test_try_from_valid_data() {
        let data = make_data(255, 0, 0);
        let result = CreateDirectDistributionData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.bump, 255);
        assert_eq!(parsed.revocable, 0);
        assert_eq!(parsed.clawback_ts, 0);
    }

    #[test]
    fn test_try_from_valid_data_revocable() {
        let data = make_data(255, 1, 0);
        let result = CreateDirectDistributionData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.revocable, 1);
    }

    #[test]
    fn test_try_from_valid_data_with_clawback_ts() {
        let data = make_data(255, 0, 1700000000);
        let result = CreateDirectDistributionData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.clawback_ts, 1700000000);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 9]; // need 10
        let result = CreateDirectDistributionData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = make_data(255, 0, 0);
        let parsed = CreateDirectDistributionData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_validate_revocable_bitmask_values() {
        for revocable in [0, 1, 2, 3, 255] {
            let data = make_data(255, revocable, 0);
            let parsed = CreateDirectDistributionData::try_from(&data[..]).unwrap();
            assert!(parsed.validate().is_ok());
            assert_eq!(parsed.revocable, revocable);
        }
    }
}
