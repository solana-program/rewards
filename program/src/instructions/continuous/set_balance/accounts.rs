use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{verify_current_program_account, verify_signer, verify_writable},
};

pub struct SetBalanceAccounts<'a> {
    pub authority: &'a AccountView,
    pub reward_pool: &'a AccountView,
    pub user_reward_account: &'a AccountView,
    pub user: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for SetBalanceAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, reward_pool, user_reward_account, user] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_signer(authority, false)?;

        verify_writable(reward_pool, true)?;
        verify_writable(user_reward_account, true)?;

        verify_current_program_account(reward_pool)?;
        verify_current_program_account(user_reward_account)?;

        Ok(Self { authority, reward_pool, user_reward_account, user })
    }
}

impl<'a> InstructionAccounts<'a> for SetBalanceAccounts<'a> {}
