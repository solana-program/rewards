use super::{AddVestingRecipientAccounts, AddVestingRecipientData};
use crate::{impl_instruction, traits::Instruction};

/// AddVestingRecipient instruction combining accounts and data
pub struct AddVestingRecipient<'a> {
    pub accounts: AddVestingRecipientAccounts<'a>,
    pub data: AddVestingRecipientData,
}

impl_instruction!(AddVestingRecipient, AddVestingRecipientAccounts, AddVestingRecipientData);

impl<'a> Instruction<'a> for AddVestingRecipient<'a> {
    type Accounts = AddVestingRecipientAccounts<'a>;
    type Data = AddVestingRecipientData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
