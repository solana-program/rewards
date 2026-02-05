use super::{CloseDirectRecipientAccounts, CloseDirectRecipientData};
use crate::{impl_instruction, traits::Instruction};

pub struct CloseDirectRecipient<'a> {
    pub accounts: CloseDirectRecipientAccounts<'a>,
    pub data: CloseDirectRecipientData,
}

impl_instruction!(CloseDirectRecipient, CloseDirectRecipientAccounts, CloseDirectRecipientData);

impl<'a> Instruction<'a> for CloseDirectRecipient<'a> {
    type Accounts = CloseDirectRecipientAccounts<'a>;
    type Data = CloseDirectRecipientData;

    #[inline(always)]
    fn accounts(&self) -> &Self::Accounts {
        &self.accounts
    }

    #[inline(always)]
    fn data(&self) -> &Self::Data {
        &self.data
    }
}
