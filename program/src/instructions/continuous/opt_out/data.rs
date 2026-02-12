use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

pub struct OptOutData;

impl<'a> TryFrom<&'a [u8]> for OptOutData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

impl<'a> InstructionData<'a> for OptOutData {
    const LEN: usize = 0;
}
