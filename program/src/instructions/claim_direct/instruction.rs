use super::{ClaimDirectAccounts, ClaimDirectData};
use crate::{impl_instruction, traits::Instruction};

/// ClaimDirect instruction combining accounts and data
pub struct ClaimDirect<'a> {
    pub accounts: ClaimDirectAccounts<'a>,
    pub data: ClaimDirectData,
}

impl_instruction!(ClaimDirect, ClaimDirectAccounts, ClaimDirectData);

impl<'a> Instruction<'a> for ClaimDirect<'a> {
    type Accounts = ClaimDirectAccounts<'a>;
    type Data = ClaimDirectData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
