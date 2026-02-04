use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

/// Instruction data for ClaimVesting
///
/// This instruction has no data - all information comes from accounts.
pub struct ClaimVestingData {}

impl<'a> TryFrom<&'a [u8]> for ClaimVestingData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl<'a> InstructionData<'a> for ClaimVestingData {
    const LEN: usize = 0;
}
