use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct OptInData {
    pub bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for OptInData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);
        Ok(Self { bump: data[0] })
    }
}

impl<'a> InstructionData<'a> for OptInData {
    const LEN: usize = 1;
}
