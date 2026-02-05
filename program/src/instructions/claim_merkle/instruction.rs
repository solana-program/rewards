use super::{ClaimMerkleAccounts, ClaimMerkleData};
use crate::{impl_instruction, traits::Instruction};

/// ClaimMerkle instruction combining accounts and data
pub struct ClaimMerkle<'a> {
    pub accounts: ClaimMerkleAccounts<'a>,
    pub data: ClaimMerkleData,
}

impl_instruction!(ClaimMerkle, ClaimMerkleAccounts, ClaimMerkleData);

impl<'a> Instruction<'a> for ClaimMerkle<'a> {
    type Accounts = ClaimMerkleAccounts<'a>;
    type Data = ClaimMerkleData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
