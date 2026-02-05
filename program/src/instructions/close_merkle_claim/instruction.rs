use super::{CloseMerkleClaimAccounts, CloseMerkleClaimData};
use crate::{impl_instruction, traits::Instruction};

/// CloseMerkleClaim instruction combining accounts and data
pub struct CloseMerkleClaim<'a> {
    pub accounts: CloseMerkleClaimAccounts<'a>,
    pub data: CloseMerkleClaimData,
}

impl_instruction!(CloseMerkleClaim, CloseMerkleClaimAccounts, CloseMerkleClaimData);

impl<'a> Instruction<'a> for CloseMerkleClaim<'a> {
    type Accounts = CloseMerkleClaimAccounts<'a>;
    type Data = CloseMerkleClaimData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
