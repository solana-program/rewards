use solana_sdk::signer::Signer;
use spl_token_2022::extension::ExtensionType;

use crate::fixtures::{CreateDirectDistributionFixture, CreateDirectDistributionSetup};
use crate::utils::{
    assert_direct_distribution, assert_rewards_error, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, test_wrong_system_program, RewardsError, TestContext,
};

#[test]
fn test_create_direct_distribution_missing_authority_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateDirectDistributionFixture>(&mut ctx, 1, 0);
}

#[test]
fn test_create_direct_distribution_missing_seeds_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<CreateDirectDistributionFixture>(&mut ctx, 2, 1);
}

#[test]
fn test_create_direct_distribution_distribution_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateDirectDistributionFixture>(&mut ctx, 3);
}

#[test]
fn test_create_direct_distribution_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateDirectDistributionFixture>(&mut ctx, 5);
}

#[test]
fn test_create_direct_distribution_authority_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<CreateDirectDistributionFixture>(&mut ctx, 6);
}

#[test]
fn test_create_direct_distribution_wrong_system_program() {
    let mut ctx = TestContext::new();
    test_wrong_system_program::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<CreateDirectDistributionFixture>(&mut ctx);
}

#[test]
fn test_create_direct_distribution_success() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_direct_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        setup.bump,
    );
}

#[test]
fn test_create_direct_distribution_success_token_2022() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new_token_2022(&mut ctx);
    let instruction = setup.build_instruction(&ctx);

    instruction.send_expect_success(&mut ctx);

    assert_direct_distribution(
        &ctx,
        &setup.distribution_pda,
        &setup.authority.pubkey(),
        &setup.mint.pubkey(),
        setup.bump,
    );
}

#[test]
fn test_create_direct_distribution_zero_amount() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::builder(&mut ctx).amount(0).build();

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::InvalidAmount);
}

#[test]
fn test_create_direct_distribution_funds_vault() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new(&mut ctx);
    let amount = setup.amount;

    let vault_balance_before = ctx.get_token_balance(&setup.vault);
    assert_eq!(vault_balance_before, 0);

    let instruction = setup.build_instruction(&ctx);
    instruction.send_expect_success(&mut ctx);

    let vault_balance_after = ctx.get_token_balance(&setup.vault);
    assert_eq!(vault_balance_after, amount);
}

#[test]
fn test_create_direct_distribution_rejects_permanent_delegate() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new_with_extension(&mut ctx, ExtensionType::PermanentDelegate);

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::PermanentDelegateNotAllowed);
}

#[test]
fn test_create_direct_distribution_rejects_non_transferable() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new_with_extension(&mut ctx, ExtensionType::NonTransferable);

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::NonTransferableNotAllowed);
}

#[test]
fn test_create_direct_distribution_rejects_pausable() {
    let mut ctx = TestContext::new();
    let setup = CreateDirectDistributionSetup::new_with_extension(&mut ctx, ExtensionType::Pausable);

    let instruction = setup.build_instruction(&ctx);
    let error = instruction.send_expect_error(&mut ctx);

    assert_rewards_error(error, RewardsError::PausableNotAllowed);
}
