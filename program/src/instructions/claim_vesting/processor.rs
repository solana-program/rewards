use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::ClaimedEvent,
    state::{VestingDistribution, VestingRecipient},
    traits::{AccountSerialize, EventSerialize},
    utils::{emit_event, get_current_timestamp, get_mint_decimals},
    ID,
};

use super::ClaimVesting;

pub fn process_claim_vesting(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClaimVesting::try_from((instruction_data, accounts))?;

    let current_ts = get_current_timestamp()?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = VestingDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    let recipient_data = ix.accounts.recipient_account.try_borrow()?;
    let mut recipient = VestingRecipient::from_account(&recipient_data, ix.accounts.recipient_account, &ID)?;
    drop(recipient_data);

    recipient.validate_distribution(ix.accounts.distribution.address())?;
    recipient.validate_recipient(ix.accounts.recipient.address())?;

    let unlocked_amount = recipient.calculate_unlocked_amount(current_ts)?;
    let claimable_amount = recipient.claimable_amount(unlocked_amount);

    if claimable_amount == 0 {
        return Err(RewardsProgramError::NothingToClaim.into());
    }

    recipient.claimed_amount =
        recipient.claimed_amount.checked_add(claimable_amount).ok_or(RewardsProgramError::MathOverflow)?;

    distribution.total_claimed =
        distribution.total_claimed.checked_add(claimable_amount).ok_or(RewardsProgramError::MathOverflow)?;

    let mut recipient_data = ix.accounts.recipient_account.try_borrow_mut()?;
    recipient.write_to_slice(&mut recipient_data)?;
    drop(recipient_data);

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    distribution.with_signer(|signers| {
        TransferChecked {
            from: ix.accounts.vault,
            mint: ix.accounts.mint,
            to: ix.accounts.recipient_token_account,
            authority: ix.accounts.distribution,
            amount: claimable_amount,
            decimals,
            token_program: ix.accounts.token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    let event =
        ClaimedEvent::new(*ix.accounts.distribution.address(), *ix.accounts.recipient.address(), claimable_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
