use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct CloseRewardPoolData;

impl<'a> TryFrom<&'a [u8]> for CloseRewardPoolData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for CloseRewardPoolData {
    const LEN: usize = 0;
}
