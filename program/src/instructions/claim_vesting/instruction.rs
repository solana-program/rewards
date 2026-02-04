use super::{ClaimVestingAccounts, ClaimVestingData};
use crate::{impl_instruction, traits::Instruction};

/// ClaimVesting instruction combining accounts and data
pub struct ClaimVesting<'a> {
    pub accounts: ClaimVestingAccounts<'a>,
    pub data: ClaimVestingData,
}

impl_instruction!(ClaimVesting, ClaimVestingAccounts, ClaimVestingData);

impl<'a> Instruction<'a> for ClaimVesting<'a> {
    type Accounts = ClaimVestingAccounts<'a>;
    type Data = ClaimVestingData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
