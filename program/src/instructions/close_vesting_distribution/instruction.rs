use super::{CloseVestingDistributionAccounts, CloseVestingDistributionData};
use crate::{impl_instruction, traits::Instruction};

/// CloseVestingDistribution instruction combining accounts and data
pub struct CloseVestingDistribution<'a> {
    pub accounts: CloseVestingDistributionAccounts<'a>,
    pub data: CloseVestingDistributionData,
}

impl_instruction!(CloseVestingDistribution, CloseVestingDistributionAccounts, CloseVestingDistributionData);

impl<'a> Instruction<'a> for CloseVestingDistribution<'a> {
    type Accounts = CloseVestingDistributionAccounts<'a>;
    type Data = CloseVestingDistributionData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
