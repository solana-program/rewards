use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

/// Instruction data for CloseVestingDistribution
///
/// This instruction has no data - all information comes from accounts.
pub struct CloseVestingDistributionData {}

impl<'a> TryFrom<&'a [u8]> for CloseVestingDistributionData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl<'a> InstructionData<'a> for CloseVestingDistributionData {
    const LEN: usize = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_empty_data() {
        let data: [u8; 0] = [];
        let result = CloseVestingDistributionData::try_from(&data[..]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_from_extra_data() {
        let data = [1, 2, 3];
        let result = CloseVestingDistributionData::try_from(&data[..]);
        assert!(result.is_ok());
    }
}
