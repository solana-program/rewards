use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::OptInEvent,
    state::{RevocationSeeds, RewardPool, UserRewardAccount, UserRewardAccountSeeds},
    traits::{AccountSerialize, AccountSize, EventSerialize, PdaSeeds},
    utils::{create_pda_account, emit_event, get_token_account_balance, is_pda_uninitialized, BalanceSource},
    ID,
};

use super::OptIn;

pub fn process_opt_in(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = OptIn::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_tracked_mint(ix.accounts.tracked_mint.address())?;

    let revocation_seeds =
        RevocationSeeds { parent: *ix.accounts.reward_pool.address(), user: *ix.accounts.user.address() };
    revocation_seeds.validate_pda_address(ix.accounts.revocation_account, &ID)?;

    if !is_pda_uninitialized(ix.accounts.revocation_account) {
        return Err(RewardsProgramError::UserRevoked.into());
    }

    let initial_balance = if pool.balance_source == BalanceSource::OnChain {
        get_token_account_balance(ix.accounts.user_tracked_token_account)?
    } else {
        0
    };

    let user_account = UserRewardAccount::new(ix.data.bump, pool.reward_per_token, initial_balance);

    let user_seeds =
        UserRewardAccountSeeds { reward_pool: *ix.accounts.reward_pool.address(), user: *ix.accounts.user.address() };
    user_seeds.validate_pda(ix.accounts.user_reward_account, &ID, ix.data.bump)?;

    let bump_seed = [ix.data.bump];
    let pda_seeds = user_seeds.seeds_with_bump(&bump_seed);
    let pda_seeds_array: [_; 4] = pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        UserRewardAccount::LEN,
        &ID,
        ix.accounts.user_reward_account,
        pda_seeds_array,
    )?;

    pool.opted_in_supply =
        pool.opted_in_supply.checked_add(initial_balance).ok_or(RewardsProgramError::MathOverflow)?;

    let mut user_data = ix.accounts.user_reward_account.try_borrow_mut()?;
    user_account.write_to_slice(&mut user_data)?;
    drop(user_data);

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    let event = OptInEvent::new(*ix.accounts.reward_pool.address(), *ix.accounts.user.address(), initial_balance);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
