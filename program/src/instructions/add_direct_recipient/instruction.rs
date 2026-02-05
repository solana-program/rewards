use super::{AddDirectRecipientAccounts, AddDirectRecipientData};
use crate::{impl_instruction, traits::Instruction};

/// AddDirectRecipient instruction combining accounts and data
pub struct AddDirectRecipient<'a> {
    pub accounts: AddDirectRecipientAccounts<'a>,
    pub data: AddDirectRecipientData,
}

impl_instruction!(AddDirectRecipient, AddDirectRecipientAccounts, AddDirectRecipientData);

impl<'a> Instruction<'a> for AddDirectRecipient<'a> {
    type Accounts = AddDirectRecipientAccounts<'a>;
    type Data = AddDirectRecipientData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
