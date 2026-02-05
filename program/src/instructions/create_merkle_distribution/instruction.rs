use super::{CreateMerkleDistributionAccounts, CreateMerkleDistributionData};
use crate::{impl_instruction, traits::Instruction};

/// CreateMerkleDistribution instruction combining accounts and data
pub struct CreateMerkleDistribution<'a> {
    pub accounts: CreateMerkleDistributionAccounts<'a>,
    pub data: CreateMerkleDistributionData,
}

impl_instruction!(CreateMerkleDistribution, CreateMerkleDistributionAccounts, CreateMerkleDistributionData);

impl<'a> Instruction<'a> for CreateMerkleDistribution<'a> {
    type Accounts = CreateMerkleDistributionAccounts<'a>;
    type Data = CreateMerkleDistributionData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
