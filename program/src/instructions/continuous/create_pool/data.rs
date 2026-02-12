use pinocchio::error::ProgramError;

use crate::{require_len, traits::InstructionData, utils::BalanceSource};

pub struct CreateRewardPoolData {
    pub bump: u8,
    pub balance_source: BalanceSource,
    pub clawback_ts: i64,
}

impl<'a> TryFrom<&'a [u8]> for CreateRewardPoolData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_len!(data, Self::LEN);

        let bump = data[0];
        let balance_source = BalanceSource::try_from(data[1])?;
        let clawback_ts = i64::from_le_bytes(data[2..10].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Self { bump, balance_source, clawback_ts })
    }
}

impl<'a> InstructionData<'a> for CreateRewardPoolData {
    const LEN: usize = 10; // bump(1) + balance_source(1) + clawback_ts(8)
}
