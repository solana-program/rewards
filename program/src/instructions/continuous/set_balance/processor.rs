use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    state::{RewardPool, UserRewardAccount},
    traits::AccountSerialize,
    utils::{sync_user_balance, update_user_rewards, BalanceSource},
    ID,
};

use super::SetBalance;

pub fn process_set_balance(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = SetBalance::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_authority(ix.accounts.authority.address())?;

    if pool.balance_source != BalanceSource::AuthoritySet {
        return Err(RewardsProgramError::BalanceSourceMismatch.into());
    }

    let user_data = ix.accounts.user_reward_account.try_borrow()?;
    let mut user = UserRewardAccount::from_account(
        &user_data,
        ix.accounts.user_reward_account,
        &ID,
        ix.accounts.reward_pool.address(),
        ix.accounts.user.address(),
    )?;
    drop(user_data);

    update_user_rewards(&pool, &mut user)?;
    sync_user_balance(&mut pool, &mut user, ix.data.balance)?;

    let mut user_data = ix.accounts.user_reward_account.try_borrow_mut()?;
    user.write_to_slice(&mut user_data)?;
    drop(user_data);

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    Ok(())
}
