use solana_sdk::signature::Signer;

use crate::fixtures::{ClaimMerkleSetup, CloseMerkleClaimFixture, CloseMerkleClaimSetup, CloseMerkleDistributionSetup};
use solana_sdk::instruction::InstructionError;

use crate::utils::{
    assert_account_closed, assert_instruction_error, test_empty_data, test_missing_signer, test_not_writable,
    test_wrong_current_program, TestContext,
};

#[test]
fn test_close_merkle_claim_missing_claimant_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CloseMerkleClaimFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_close_merkle_claim_claim_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CloseMerkleClaimFixture>(&mut ctx, 2);
}

#[test]
fn test_close_merkle_claim_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CloseMerkleClaimFixture>(&mut ctx);
}

#[test]
fn test_close_merkle_claim_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CloseMerkleClaimFixture>(&mut ctx);
}

#[test]
fn test_close_merkle_claim_success() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleClaimSetup::new(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.claim_pda);
}

#[test]
fn test_close_merkle_claim_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleClaimSetup::new_token_2022(&mut ctx);

    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.claim_pda);
}

#[test]
fn test_close_merkle_claim_distribution_not_closed() {
    let mut ctx = TestContext::new();

    // Create claim setup without closing distribution
    let claim_setup = ClaimMerkleSetup::builder(&mut ctx).linear().build();

    // Make a partial claim so claim account exists but distribution is still open
    let claim_ix = claim_setup.build_instruction_with_amount(&ctx, claim_setup.total_amount / 2);
    claim_ix.send_expect_success(&mut ctx);

    // Try to close claim while distribution still open
    let close_setup = CloseMerkleClaimSetup {
        claimant: claim_setup.claimant.insecure_clone(),
        distribution_pda: claim_setup.distribution_pda,
        claim_pda: claim_setup.claim_pda,
        token_program: claim_setup.token_program,
    };

    let test_ix = close_setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);

    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}

#[test]
fn test_close_merkle_claim_wrong_claimant() {
    let mut ctx = TestContext::new();
    let setup = CloseMerkleClaimSetup::new(&mut ctx);

    let wrong_claimant = ctx.create_funded_keypair();

    let test_ix = setup.build_instruction_with_wrong_claimant(&ctx, &wrong_claimant);
    let error = test_ix.send_expect_error(&mut ctx);

    // Should fail because the wrong claimant's claim PDA doesn't exist (owned by system)
    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}

#[test]
fn test_close_merkle_claim_returns_rent() {
    let mut ctx = TestContext::new();

    // Create claim with partial vesting and make a partial claim
    let claim_setup = ClaimMerkleSetup::builder(&mut ctx).linear().warp_to_end(false).build();

    // Warp to 50% and make a partial claim
    let mid_point = claim_setup.start_ts() + (claim_setup.end_ts() - claim_setup.start_ts()) / 2;
    ctx.warp_to_timestamp(mid_point);

    let claim_ix = claim_setup.build_instruction(&ctx);
    claim_ix.send_expect_success(&mut ctx);

    // Record claimant balance before close
    let claimant_sol_before = ctx.get_account(&claim_setup.claimant.pubkey()).map(|a| a.lamports).unwrap_or(0);

    // Get claim account rent
    let claim_account = ctx.get_account(&claim_setup.claim_pda).expect("Claim should exist");
    let claim_rent = claim_account.lamports;

    // Warp to clawback and close distribution
    ctx.warp_to_timestamp(claim_setup.end_ts() + 86400 * 365 + 1);

    let close_dist_setup = CloseMerkleDistributionSetup {
        authority: claim_setup.authority.insecure_clone(),
        distribution_pda: claim_setup.distribution_pda,
        mint: claim_setup.mint,
        vault: claim_setup.vault,
        authority_token_account: ctx.create_token_account(&claim_setup.authority.pubkey(), &claim_setup.mint),
        token_program: claim_setup.token_program,
        funded_amount: 0,
        clawback_ts: claim_setup.end_ts() + 86400 * 365,
    };

    let close_dist_ix = close_dist_setup.build_instruction(&ctx);
    close_dist_ix.send_expect_success(&mut ctx);

    // Now close the claim
    let close_claim_setup = CloseMerkleClaimSetup {
        claimant: claim_setup.claimant.insecure_clone(),
        distribution_pda: claim_setup.distribution_pda,
        claim_pda: claim_setup.claim_pda,
        token_program: claim_setup.token_program,
    };

    let close_claim_ix = close_claim_setup.build_instruction(&ctx);
    close_claim_ix.send_expect_success(&mut ctx);

    // Check claimant received rent back
    let claimant_sol_after = ctx.get_account(&claim_setup.claimant.pubkey()).map(|a| a.lamports).unwrap_or(0);
    assert_eq!(claimant_sol_after, claimant_sol_before + claim_rent);
}
