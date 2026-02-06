use rewards_program_client::types::VestingSchedule;
use solana_sdk::signature::Signer;

use crate::fixtures::{AddDirectRecipientSetup, ClaimMerkleSetup};
use crate::utils::{assert_direct_recipient, assert_rewards_error, RewardsError, TestContext};

// ── Cliff (type=2) via Direct Distribution ──

#[test]
fn test_cliff_direct_nothing_before_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let cliff_ts = current_ts + 1000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx).schedule(VestingSchedule::Cliff { cliff_ts }).build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    let test_ix = claim_setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_cliff_direct_full_at_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let cliff_ts = current_ts + 1000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx).schedule(VestingSchedule::Cliff { cliff_ts }).build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    ctx.warp_to_timestamp(cliff_ts);
    let test_ix = claim_setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&claim_setup.recipient_token_account);
    assert_eq!(balance, setup.amount);
}

// ── CliffLinear (type=3) via Direct Distribution ──

#[test]
fn test_cliff_linear_direct_nothing_before_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    let test_ix = claim_setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_cliff_linear_direct_accumulated_at_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    ctx.warp_to_timestamp(cliff_ts);
    let test_ix = claim_setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&claim_setup.recipient_token_account);
    let expected = setup.amount / 4; // 1000/4000 = 25%
    assert!(balance >= expected - 1 && balance <= expected + 1, "Expected ~{expected}, got {balance}");
}

#[test]
fn test_cliff_linear_direct_50_percent() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    ctx.warp_to_timestamp(start_ts + 2000);
    let test_ix = claim_setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&claim_setup.recipient_token_account);
    let expected = setup.amount / 2; // 2000/4000 = 50%
    assert!(balance >= expected - 1 && balance <= expected + 1, "Expected ~{expected}, got {balance}");
}

#[test]
fn test_cliff_linear_direct_full_at_end() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    ctx.warp_to_timestamp(end_ts);
    let test_ix = claim_setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&claim_setup.recipient_token_account);
    assert_eq!(balance, setup.amount);
}

#[test]
fn test_cliff_linear_direct_multiple_claims() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .build();

    let claim_setup = crate::fixtures::ClaimDirectSetup::from_recipient_setup(&mut ctx, &setup, false);

    // First claim at cliff (25%)
    ctx.warp_to_timestamp(cliff_ts);
    let test_ix1 = claim_setup.build_instruction(&ctx);
    test_ix1.send_expect_success(&mut ctx);

    let balance_first = ctx.get_token_balance(&claim_setup.recipient_token_account);
    let expected_first = setup.amount / 4;
    assert!(
        balance_first >= expected_first - 1 && balance_first <= expected_first + 1,
        "Expected ~{expected_first}, got {balance_first}"
    );

    // Second claim at end (remaining 75%)
    ctx.warp_to_timestamp(end_ts);
    let test_ix2 = claim_setup.build_instruction(&ctx);
    test_ix2.send_expect_success(&mut ctx);

    let balance_total = ctx.get_token_balance(&claim_setup.recipient_token_account);
    assert_eq!(balance_total, setup.amount);
}

// ── CliffLinear via Merkle Distribution ──

#[test]
fn test_cliff_linear_merkle_nothing_before_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = ClaimMerkleSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .warp_to_end(false)
        .build();

    let test_ix = setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_cliff_linear_merkle_accumulated_at_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = ClaimMerkleSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .warp_to_end(false)
        .build();

    ctx.warp_to_timestamp(cliff_ts);
    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    let expected = setup.total_amount / 4;
    assert!(balance >= expected - 1 && balance <= expected + 1, "Expected ~{expected}, got {balance}");
}

#[test]
fn test_cliff_linear_merkle_full_at_end() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = ClaimMerkleSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .warp_to_end(false)
        .build();

    ctx.warp_to_timestamp(end_ts);
    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance, setup.total_amount);
}

// ── Cliff via Merkle Distribution ──

#[test]
fn test_cliff_merkle_nothing_before_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let cliff_ts = current_ts + 1000;

    let setup =
        ClaimMerkleSetup::builder(&mut ctx).schedule(VestingSchedule::Cliff { cliff_ts }).warp_to_end(false).build();

    let test_ix = setup.build_instruction(&ctx);
    let error = test_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_cliff_merkle_full_at_cliff() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let cliff_ts = current_ts + 1000;

    let setup =
        ClaimMerkleSetup::builder(&mut ctx).schedule(VestingSchedule::Cliff { cliff_ts }).warp_to_end(false).build();

    ctx.warp_to_timestamp(cliff_ts);
    let test_ix = setup.build_instruction(&ctx);
    test_ix.send_expect_success(&mut ctx);

    let balance = ctx.get_token_balance(&setup.claimant_token_account);
    assert_eq!(balance, setup.total_amount);
}

// ── Validation Tests ──

#[test]
fn test_cliff_linear_invalid_cliff_before_start() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear {
            start_ts: current_ts + 1000,
            cliff_ts: current_ts + 500, // cliff before start
            end_ts: current_ts + 4000,
        })
        .build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidCliffTimestamp);
}

#[test]
fn test_cliff_linear_invalid_cliff_after_end() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear {
            start_ts: current_ts,
            cliff_ts: current_ts + 5000, // cliff after end
            end_ts: current_ts + 4000,
        })
        .build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidCliffTimestamp);
}

#[test]
fn test_cliff_schedule_zero_cliff_ts_invalid() {
    let mut ctx = TestContext::new();

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::Cliff { cliff_ts: 0 }) // cliff_ts must be > 0 for Cliff type
        .build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidCliffTimestamp);
}

// ── Account State ──

#[test]
fn test_cliff_linear_direct_account_state() {
    let mut ctx = TestContext::new();
    let current_ts = ctx.get_current_timestamp();
    let start_ts = current_ts;
    let cliff_ts = current_ts + 1000;
    let end_ts = current_ts + 4000;

    let setup = AddDirectRecipientSetup::builder(&mut ctx)
        .schedule(VestingSchedule::CliffLinear { start_ts, cliff_ts, end_ts })
        .build();

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    assert_direct_recipient(
        &ctx,
        &setup.recipient_pda,
        &setup.recipient.pubkey(),
        setup.amount,
        0,
        setup.recipient_bump,
    );
}
