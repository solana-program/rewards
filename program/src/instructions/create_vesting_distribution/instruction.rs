use super::{CreateVestingDistributionAccounts, CreateVestingDistributionData};
use crate::{impl_instruction, traits::Instruction};

/// CreateVestingDistribution instruction combining accounts and data
pub struct CreateVestingDistribution<'a> {
    pub accounts: CreateVestingDistributionAccounts<'a>,
    pub data: CreateVestingDistributionData,
}

impl_instruction!(CreateVestingDistribution, CreateVestingDistributionAccounts, CreateVestingDistributionData);

impl<'a> Instruction<'a> for CreateVestingDistribution<'a> {
    type Accounts = CreateVestingDistributionAccounts<'a>;
    type Data = CreateVestingDistributionData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
