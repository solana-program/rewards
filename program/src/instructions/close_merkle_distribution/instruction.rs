use super::{CloseMerkleDistributionAccounts, CloseMerkleDistributionData};
use crate::{impl_instruction, traits::Instruction};

/// CloseMerkleDistribution instruction combining accounts and data
pub struct CloseMerkleDistribution<'a> {
    pub accounts: CloseMerkleDistributionAccounts<'a>,
    pub data: CloseMerkleDistributionData,
}

impl_instruction!(CloseMerkleDistribution, CloseMerkleDistributionAccounts, CloseMerkleDistributionData);

impl<'a> Instruction<'a> for CloseMerkleDistribution<'a> {
    type Accounts = CloseMerkleDistributionAccounts<'a>;
    type Data = CloseMerkleDistributionData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
