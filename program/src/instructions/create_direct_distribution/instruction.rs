use super::{CreateDirectDistributionAccounts, CreateDirectDistributionData};
use crate::{impl_instruction, traits::Instruction};

/// CreateDirectDistribution instruction combining accounts and data
pub struct CreateDirectDistribution<'a> {
    pub accounts: CreateDirectDistributionAccounts<'a>,
    pub data: CreateDirectDistributionData,
}

impl_instruction!(CreateDirectDistribution, CreateDirectDistributionAccounts, CreateDirectDistributionData);

impl<'a> Instruction<'a> for CreateDirectDistribution<'a> {
    type Accounts = CreateDirectDistributionAccounts<'a>;
    type Data = CreateDirectDistributionData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
