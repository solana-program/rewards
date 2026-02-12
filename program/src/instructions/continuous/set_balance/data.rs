use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData};

pub struct SetBalanceData {
    pub balance: u64,
}

impl<'a> TryFrom<&'a [u8]> for SetBalanceData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let balance = u64::from_le_bytes(data[0..8].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { balance })
    }
}

impl<'a> InstructionData<'a> for SetBalanceData {
    const LEN: usize = 8;
}
