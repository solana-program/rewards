use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    events::ClaimedEvent,
    state::{MerkleClaim, MerkleClaimSeeds, MerkleDistribution},
    traits::{
        AccountParse, AccountSerialize, AccountSize, ClaimTracker, Distribution, DistributionSigner, EventSerialize,
        PdaSeeds, VestingParams,
    },
    utils::{
        close_pda_account, compute_leaf_hash, create_pda_account_idempotent, emit_event, get_current_timestamp,
        get_mint_decimals, is_pda_uninitialized, resolve_claim_amount, verify_proof_or_error,
    },
    ID,
};

use super::ClaimMerkle;

pub fn process_claim_merkle(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    let ix = ClaimMerkle::try_from((instruction_data, accounts))?;

    let current_ts = get_current_timestamp()?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let mut distribution = MerkleDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    drop(distribution_data);

    let schedule_bytes = ix.data.schedule.to_bytes();
    let leaf = compute_leaf_hash(ix.accounts.claimant.address(), ix.data.total_amount, &schedule_bytes);
    verify_proof_or_error(&ix.data.proof, &distribution.merkle_root, &leaf)?;

    let claim_seeds = MerkleClaimSeeds {
        distribution: *ix.accounts.distribution.address(),
        claimant: *ix.accounts.claimant.address(),
    };

    claim_seeds.validate_pda(ix.accounts.claim_account, &ID, ix.data.claim_bump)?;

    let claim_bump_seed = [ix.data.claim_bump];
    let claim_pda_seeds = claim_seeds.seeds_with_bump(&claim_bump_seed);
    let claim_pda_seeds_array: [_; 4] = claim_pda_seeds.try_into().map_err(|_| ProgramError::InvalidArgument)?;

    let is_new_claim = is_pda_uninitialized(ix.accounts.claim_account);

    create_pda_account_idempotent(
        ix.accounts.payer,
        MerkleClaim::LEN,
        &ID,
        ix.accounts.claim_account,
        claim_pda_seeds_array,
    )?;

    let mut claim = if is_new_claim {
        let claim = MerkleClaim::new(ix.data.claim_bump);
        let mut claim_data = ix.accounts.claim_account.try_borrow_mut()?;
        claim.write_to_slice(&mut claim_data)?;
        drop(claim_data);
        claim
    } else {
        let claim_data = ix.accounts.claim_account.try_borrow()?;
        let claim = MerkleClaim::parse_from_bytes(&claim_data)?;
        drop(claim_data);
        claim
    };

    let unlocked_amount = VestingParams::calculate_unlocked(&ix.data, current_ts)?;
    let claimable_amount = ClaimTracker::claimable_amount(&claim, unlocked_amount)?;
    let claim_amount = resolve_claim_amount(ix.data.amount, claimable_amount)?;

    ClaimTracker::add_claimed(&mut claim, claim_amount)?;
    Distribution::add_claimed(&mut distribution, claim_amount)?;

    let mut claim_data = ix.accounts.claim_account.try_borrow_mut()?;
    claim.write_to_slice(&mut claim_data)?;
    drop(claim_data);

    let mut distribution_data = ix.accounts.distribution.try_borrow_mut()?;
    distribution.write_to_slice(&mut distribution_data)?;
    drop(distribution_data);

    let decimals = get_mint_decimals(ix.accounts.mint)?;

    distribution.with_signer(|signers| {
        TransferChecked {
            from: ix.accounts.vault,
            mint: ix.accounts.mint,
            to: ix.accounts.claimant_token_account,
            authority: ix.accounts.distribution,
            amount: claim_amount,
            decimals,
            token_program: ix.accounts.token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    let event = ClaimedEvent::new(*ix.accounts.distribution.address(), *ix.accounts.claimant.address(), claim_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    // Close the claim account if the total amount has been claimed
    if ClaimTracker::claimed_amount(&claim) >= ix.data.total_amount {
        close_pda_account(ix.accounts.claim_account, ix.accounts.payer)?;
    }

    Ok(())
}
