use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::VestingRecipientAddedEvent,
    state::{VestingDistribution, VestingRecipient},
    traits::{AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, get_token_account_balance, VestingScheduleType},
    ID,
};

use super::AddVestingRecipient;

pub fn process_add_vesting_recipient(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = AddVestingRecipient::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let schedule_type =
        VestingScheduleType::from_u8(ix.data.schedule_type).ok_or(RewardsProgramError::InvalidScheduleType)?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = VestingDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    distribution.validate_authority(ix.accounts.authority.address())?;

    let new_total_allocated =
        distribution.total_allocated.checked_add(ix.data.amount).ok_or(RewardsProgramError::MathOverflow)?;

    let vault_balance = get_token_account_balance(ix.accounts.vault)?;
    if new_total_allocated > vault_balance {
        return Err(RewardsProgramError::InsufficientFunds.into());
    }

    let recipient = VestingRecipient::new(
        ix.data.bump,
        *ix.accounts.distribution.address(),
        *ix.accounts.recipient.address(),
        ix.data.amount,
        schedule_type,
        ix.data.start_ts,
        ix.data.end_ts,
    );

    recipient.validate_pda(ix.accounts.recipient_account, &ID, ix.data.bump)?;

    let bump_seed = [ix.data.bump];
    let recipient_seeds = recipient.seeds_with_bump(&bump_seed);
    let recipient_seeds_array: [_; 4] = recipient_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        VestingRecipient::LEN,
        &ID,
        ix.accounts.recipient_account,
        recipient_seeds_array,
    )?;

    let mut recipient_data = ix.accounts.recipient_account.try_borrow_mut()?;
    recipient.write_to_slice(&mut recipient_data)?;
    drop(recipient_data);

    distribution.total_allocated = new_total_allocated;
    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let event = VestingRecipientAddedEvent::new(
        *ix.accounts.distribution.address(),
        *ix.accounts.recipient.address(),
        ix.data.amount,
        ix.data.schedule_type,
        ix.data.start_ts,
        ix.data.end_ts,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
