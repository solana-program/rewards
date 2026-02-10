use rewards_program_client::accounts::DirectDistribution;
use rewards_program_client::types::{RevokeMode, VestingSchedule};
use solana_sdk::signature::Signer;

use crate::fixtures::{CreateDirectDistributionSetup, RevokeDirectRecipientFixture, RevokeDirectRecipientSetup};
use crate::utils::{
    assert_account_closed, assert_rewards_error, expected_linear_unlock, test_empty_data, test_missing_signer,
    test_not_writable, test_wrong_current_program, RewardsError, TestContext, PROGRAM_ID,
};

// ── Generic fixture tests ──────────────────────────────────────────

#[test]
fn test_revoke_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<RevokeDirectRecipientFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_revoke_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeDirectRecipientFixture>(&mut ctx, 1);
}

#[test]
fn test_revoke_recipient_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeDirectRecipientFixture>(&mut ctx, 2);
}

#[test]
fn test_revoke_payer_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeDirectRecipientFixture>(&mut ctx, 4);
}

#[test]
fn test_revoke_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeDirectRecipientFixture>(&mut ctx, 6);
}

#[test]
fn test_revoke_recipient_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<RevokeDirectRecipientFixture>(&mut ctx, 7);
}

#[test]
fn test_revoke_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<RevokeDirectRecipientFixture>(&mut ctx);
}

#[test]
fn test_revoke_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<RevokeDirectRecipientFixture>(&mut ctx);
}

// ── Error paths ────────────────────────────────────────────────────

#[test]
fn test_revoke_not_revocable() {
    let mut ctx = TestContext::new();

    let distribution_setup = CreateDirectDistributionSetup::builder(&mut ctx).build();
    let create_ix = distribution_setup.build_instruction(&ctx);
    create_ix.send_expect_success(&mut ctx);

    let current_ts = ctx.get_current_timestamp();
    let recipient = ctx.create_funded_keypair();
    let rent_payer = ctx.create_funded_keypair();
    let (recipient_pda, recipient_bump) =
        crate::utils::find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

    let authority_token_account = ctx.create_ata_for_program_with_balance(
        &distribution_setup.authority.pubkey(),
        &distribution_setup.mint.pubkey(),
        1_000_000,
        &distribution_setup.token_program,
    );

    let (event_authority, _) = crate::utils::find_event_authority_pda();
    let mut add_builder = rewards_program_client::instructions::AddDirectRecipientBuilder::new();
    add_builder
        .payer(rent_payer.pubkey())
        .authority(distribution_setup.authority.pubkey())
        .distribution(distribution_setup.distribution_pda)
        .recipient_account(recipient_pda)
        .recipient(recipient.pubkey())
        .mint(distribution_setup.mint.pubkey())
        .distribution_vault(distribution_setup.distribution_vault)
        .authority_token_account(authority_token_account)
        .token_program(distribution_setup.token_program)
        .event_authority(event_authority)
        .bump(recipient_bump)
        .amount(1_000_000)
        .schedule(VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 });

    let add_ix = crate::utils::TestInstruction {
        instruction: add_builder.instruction(),
        signers: vec![rent_payer.insecure_clone(), distribution_setup.authority.insecure_clone()],
        name: "AddDirectRecipient",
    };
    add_ix.send_expect_success(&mut ctx);

    let recipient_token_account = ctx.create_ata_for_program(
        &recipient.pubkey(),
        &distribution_setup.mint.pubkey(),
        &distribution_setup.token_program,
    );

    let setup = RevokeDirectRecipientSetup {
        authority: distribution_setup.authority,
        distribution_pda: distribution_setup.distribution_pda,
        recipient,
        recipient_pda,
        payer: rent_payer,
        mint: distribution_setup.mint.pubkey(),
        distribution_vault: distribution_setup.distribution_vault,
        recipient_token_account,
        authority_token_account,
        token_program: distribution_setup.token_program,
        amount: 1_000_000,
        start_ts: current_ts,
        end_ts: current_ts + 86400 * 365,
    };

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_wrong_authority() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    let wrong_authority = ctx.create_funded_keypair();
    let revoke_ix = setup.build_instruction_with_wrong_authority(&ctx, &wrong_authority, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::UnauthorizedAuthority);
}

#[test]
fn test_revoke_invalid_mode() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    // Build a valid instruction then patch the data byte to an invalid mode
    let mut test_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    // Instruction data layout: [discriminator(1), revoke_mode(1)]
    // Patch revoke_mode byte to invalid value 2
    test_ix.instruction.data[1] = 2;
    let error = test_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::InvalidRevokeMode);
}

// ── Happy paths ────────────────────────────────────────────────────

#[test]
fn test_revoke_non_vested_at_midpoint() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let vault_balance_before = ctx.get_token_balance(&setup.distribution_vault);
    let recipient_balance_before = ctx.get_token_balance(&setup.recipient_token_account);
    let authority_balance_before = ctx.get_token_balance(&setup.authority_token_account);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let expected_vested = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, midpoint);
    let expected_unvested = setup.amount - expected_vested;

    let recipient_balance_after = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(
        recipient_balance_after,
        recipient_balance_before + expected_vested,
        "Recipient should receive vested tokens"
    );

    let authority_balance_after = ctx.get_token_balance(&setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + expected_unvested,
        "Authority should receive unvested tokens"
    );

    let vault_balance_after = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(
        vault_balance_after,
        vault_balance_before - expected_vested - expected_unvested,
        "Vault should decrease by vested + unvested"
    );

    assert_account_closed(&ctx, &setup.recipient_pda);

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    assert_eq!(dist_account.owner, PROGRAM_ID);
    let dist = DirectDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_allocated, setup.amount - expected_unvested, "total_allocated should have freed unvested");
    assert_eq!(dist.total_claimed, expected_vested, "total_claimed should include vested_unclaimed");
}

#[test]
fn test_revoke_full_at_midpoint() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let vault_balance_before = ctx.get_token_balance(&setup.distribution_vault);
    let recipient_balance_before = ctx.get_token_balance(&setup.recipient_token_account);
    let authority_balance_before = ctx.get_token_balance(&setup.authority_token_account);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);

    let expected_vested = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, midpoint);
    let expected_unvested = setup.amount - expected_vested;
    let total_freed = expected_unvested + expected_vested;

    let recipient_balance_after = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(recipient_balance_after, recipient_balance_before, "Recipient should receive nothing in Full mode");

    let authority_balance_after = ctx.get_token_balance(&setup.authority_token_account);
    assert_eq!(
        authority_balance_after,
        authority_balance_before + total_freed,
        "Authority should receive all unclaimed tokens in Full mode"
    );

    let vault_balance_after = ctx.get_token_balance(&setup.distribution_vault);
    assert_eq!(
        vault_balance_after,
        vault_balance_before - total_freed,
        "Vault should decrease by total freed in Full mode"
    );

    assert_account_closed(&ctx, &setup.recipient_pda);

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = DirectDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    let total_freed = expected_unvested + expected_vested;
    assert_eq!(dist.total_allocated, setup.amount - total_freed, "total_allocated should free all unclaimed");
    assert_eq!(dist.total_claimed, 0, "total_claimed should not change in Full mode");
}

#[test]
fn test_revoke_before_vesting_starts() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let recipient_balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(recipient_balance, 0, "Recipient should receive nothing when nothing is vested");

    assert_account_closed(&ctx, &setup.recipient_pda);

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = DirectDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_allocated, 0, "All allocation should be freed");
    assert_eq!(dist.total_claimed, 0, "Nothing claimed");
}

#[test]
fn test_revoke_after_full_vesting() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    ctx.warp_to_timestamp(setup.end_ts + 1);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let recipient_balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(recipient_balance, setup.amount, "Recipient should receive full amount when fully vested");

    assert_account_closed(&ctx, &setup.recipient_pda);

    let dist_account = ctx.get_account(&setup.distribution_pda).expect("Distribution should exist");
    let dist = DirectDistribution::from_bytes(&dist_account.data).expect("Should deserialize");
    assert_eq!(dist.total_allocated, setup.amount, "total_allocated unchanged (no unvested to free)");
    assert_eq!(dist.total_claimed, setup.amount, "All marked as claimed");
}

#[test]
fn test_revoke_with_immediate_schedule() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).schedule(VestingSchedule::Immediate).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let recipient_balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(recipient_balance, setup.amount, "All should transfer with Immediate schedule");

    assert_account_closed(&ctx, &setup.recipient_pda);
}

#[test]
fn test_revoke_with_token_2022() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new_token_2022(&mut ctx);

    let midpoint = setup.start_ts + (setup.end_ts - setup.start_ts) / 2;
    ctx.warp_to_timestamp(midpoint);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let expected_vested = expected_linear_unlock(setup.amount, setup.start_ts, setup.end_ts, midpoint);
    let recipient_balance = ctx.get_token_balance(&setup.recipient_token_account);
    assert_eq!(recipient_balance, expected_vested, "Token-2022 revoke should work");

    assert_account_closed(&ctx, &setup.recipient_pda);
}

#[test]
fn test_revoke_rent_returned_to_payer() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    let payer_balance_before = ctx.get_account(&setup.payer.pubkey()).unwrap().lamports;

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);

    let payer_balance_after = ctx.get_account(&setup.payer.pubkey()).unwrap().lamports;
    assert!(
        payer_balance_after > payer_balance_before,
        "Payer should receive rent refund: before={}, after={}",
        payer_balance_before,
        payer_balance_after
    );
}

#[test]
fn test_revoke_freed_allocation_allows_new_recipient() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::new(&mut ctx);

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);

    assert_account_closed(&ctx, &setup.recipient_pda);

    let new_recipient = ctx.create_funded_keypair();
    let new_payer = ctx.create_funded_keypair();
    let (new_recipient_pda, new_recipient_bump) =
        crate::utils::find_direct_recipient_pda(&setup.distribution_pda, &new_recipient.pubkey());

    let authority_token_account = ctx.create_ata_for_program_with_balance(
        &setup.authority.pubkey(),
        &setup.mint,
        setup.amount,
        &setup.token_program,
    );

    let (event_authority, _) = crate::utils::find_event_authority_pda();
    let mut add_builder = rewards_program_client::instructions::AddDirectRecipientBuilder::new();
    add_builder
        .payer(new_payer.pubkey())
        .authority(setup.authority.pubkey())
        .distribution(setup.distribution_pda)
        .recipient_account(new_recipient_pda)
        .recipient(new_recipient.pubkey())
        .mint(setup.mint)
        .distribution_vault(setup.distribution_vault)
        .authority_token_account(authority_token_account)
        .token_program(setup.token_program)
        .event_authority(event_authority)
        .bump(new_recipient_bump)
        .amount(setup.amount)
        .schedule(VestingSchedule::Immediate);

    let add_ix = crate::utils::TestInstruction {
        instruction: add_builder.instruction(),
        signers: vec![new_payer.insecure_clone(), setup.authority.insecure_clone()],
        name: "AddDirectRecipient",
    };
    add_ix.send_expect_success(&mut ctx);
}

// ── Bitmask permission tests ──────────────────────────────────────

#[test]
fn test_revoke_non_vested_rejected_when_only_full_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(2).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_full_rejected_when_only_non_vested_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(1).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_both_modes_succeed_when_revocable_3() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(3).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_full_succeeds_when_revocable_3() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(3).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_non_vested_succeeds_when_only_non_vested_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(1).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_full_succeeds_when_only_full_bit_set() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(2).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    revoke_ix.send_expect_success(&mut ctx);
}

#[test]
fn test_revoke_all_modes_rejected_when_revocable_0() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(0).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::NonVested);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}

#[test]
fn test_revoke_full_rejected_when_revocable_0() {
    let mut ctx = TestContext::new();
    let setup = RevokeDirectRecipientSetup::builder(&mut ctx).revocable(0).build();

    let revoke_ix = setup.build_instruction(&ctx, RevokeMode::Full);
    let error = revoke_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::DistributionNotRevocable);
}
