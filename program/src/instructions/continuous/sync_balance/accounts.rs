use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{verify_current_program_account, verify_owned_by, verify_readonly, verify_token_program, verify_writable},
};

pub struct SyncBalanceAccounts<'a> {
    pub reward_pool: &'a AccountView,
    pub user_reward_account: &'a AccountView,
    pub user: &'a AccountView,
    pub user_tracked_token_account: &'a AccountView,
    pub tracked_mint: &'a AccountView,
    pub tracked_token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for SyncBalanceAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [reward_pool, user_reward_account, user, user_tracked_token_account, tracked_mint, tracked_token_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        verify_writable(reward_pool, true)?;
        verify_writable(user_reward_account, true)?;

        verify_readonly(user_tracked_token_account)?;
        verify_readonly(tracked_mint)?;

        verify_current_program_account(reward_pool)?;
        verify_current_program_account(user_reward_account)?;

        verify_token_program(tracked_token_program)?;

        verify_owned_by(tracked_mint, tracked_token_program.address())?;
        verify_owned_by(user_tracked_token_account, tracked_token_program.address())?;

        Ok(Self {
            reward_pool,
            user_reward_account,
            user,
            user_tracked_token_account,
            tracked_mint,
            tracked_token_program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for SyncBalanceAccounts<'a> {}
