use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_associated_token_account::instructions::CreateIdempotent;
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    events::DistributionCreatedEvent,
    state::MerkleDistribution,
    traits::{AccountSerialize, AccountSize, EventSerialize, InstructionData, PdaSeeds},
    utils::{create_pda_account, emit_event, get_mint_decimals},
    ID,
};

use super::CreateMerkleDistribution;

pub fn process_create_merkle_distribution(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CreateMerkleDistribution::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let distribution = MerkleDistribution::new(
        ix.data.bump,
        ix.data.revocable,
        *ix.accounts.authority.address(),
        *ix.accounts.mint.address(),
        *ix.accounts.seed.address(),
        ix.data.merkle_root,
        ix.data.total_amount,
        ix.data.clawback_ts,
    );

    distribution.validate_pda(ix.accounts.distribution, &ID, ix.data.bump)?;

    let bump_seed = [ix.data.bump];
    let distribution_seeds = distribution.seeds_with_bump(&bump_seed);
    let distribution_seeds_array: [_; 5] = distribution_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    create_pda_account(
        ix.accounts.payer,
        MerkleDistribution::LEN,
        &ID,
        ix.accounts.distribution,
        distribution_seeds_array,
    )?;

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    CreateIdempotent {
        funding_account: ix.accounts.payer,
        account: ix.accounts.distribution_vault,
        wallet: ix.accounts.distribution,
        mint: ix.accounts.mint,
        system_program: ix.accounts.system_program,
        token_program: ix.accounts.token_program,
    }
    .invoke()?;

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    TransferChecked {
        from: ix.accounts.authority_token_account,
        mint: ix.accounts.mint,
        to: ix.accounts.distribution_vault,
        authority: ix.accounts.authority,
        amount: ix.data.amount,
        decimals,
        token_program: ix.accounts.token_program.address(),
    }
    .invoke()?;

    let event = DistributionCreatedEvent::merkle(
        *ix.accounts.authority.address(),
        *ix.accounts.mint.address(),
        *ix.accounts.seed.address(),
        ix.data.merkle_root,
        ix.data.total_amount,
        ix.data.clawback_ts,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
