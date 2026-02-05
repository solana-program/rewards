use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

/// Instruction data for CloseMerkleDistribution (empty)
pub struct CloseMerkleDistributionData;

impl<'a> TryFrom<&'a [u8]> for CloseMerkleDistributionData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for CloseMerkleDistributionData {
    const LEN: usize = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_empty_data() {
        let data: [u8; 0] = [];
        let result = CloseMerkleDistributionData::try_from(&data[..]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_from_extra_data() {
        let data = [1, 2, 3];
        let result = CloseMerkleDistributionData::try_from(&data[..]);
        assert!(result.is_ok());
    }
}
