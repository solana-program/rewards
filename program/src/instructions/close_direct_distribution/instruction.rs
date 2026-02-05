use super::{CloseDirectDistributionAccounts, CloseDirectDistributionData};
use crate::{impl_instruction, traits::Instruction};

/// CloseDirectDistribution instruction combining accounts and data
pub struct CloseDirectDistribution<'a> {
    pub accounts: CloseDirectDistributionAccounts<'a>,
    pub data: CloseDirectDistributionData,
}

impl_instruction!(CloseDirectDistribution, CloseDirectDistributionAccounts, CloseDirectDistributionData);

impl<'a> Instruction<'a> for CloseDirectDistribution<'a> {
    type Accounts = CloseDirectDistributionAccounts<'a>;
    type Data = CloseDirectDistributionData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
