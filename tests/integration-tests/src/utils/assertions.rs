use rewards_program_client::accounts::{VestingDistribution, VestingRecipient};
use solana_sdk::{instruction::InstructionError, pubkey::Pubkey, transaction::TransactionError};

use crate::utils::{TestContext, PROGRAM_ID};

pub use rewards_program_client::errors::RewardsProgramError as RewardsError;

/// Assert that a transaction error is the expected rewards program error
pub fn assert_rewards_error(tx_error: TransactionError, expected: RewardsError) {
    assert_instruction_error(tx_error, InstructionError::Custom(expected as u32));
}

/// Assert that a transaction error contains the expected instruction error
pub fn assert_instruction_error(tx_error: TransactionError, expected: InstructionError) {
    match tx_error {
        TransactionError::InstructionError(_, err) => {
            assert_eq!(err, expected, "Expected {expected:?}, got {err:?}");
        }
        other => panic!("Expected InstructionError, got {other:?}"),
    }
}

/// Assert that a transaction error is a custom program error with the given code
pub fn assert_custom_error(tx_error: TransactionError, expected_code: u32) {
    assert_instruction_error(tx_error, InstructionError::Custom(expected_code));
}

/// Assert that a vesting distribution account exists with expected values
pub fn assert_vesting_distribution(
    ctx: &TestContext,
    distribution_pda: &Pubkey,
    expected_authority: &Pubkey,
    expected_mint: &Pubkey,
    expected_bump: u8,
) {
    let account = ctx.get_account(distribution_pda).expect("Distribution account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "Distribution should be owned by program");

    let data = VestingDistribution::from_bytes(&account.data).expect("Failed to deserialize distribution");

    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.authority, *expected_authority);
    assert_eq!(data.mint, *expected_mint);
}

/// Assert that a vesting recipient account exists with expected values
pub fn assert_vesting_recipient(
    ctx: &TestContext,
    recipient_pda: &Pubkey,
    expected_recipient: &Pubkey,
    expected_total_amount: u64,
    expected_claimed_amount: u64,
    expected_bump: u8,
) {
    let account = ctx.get_account(recipient_pda).expect("Recipient account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "Recipient should be owned by program");

    let data = VestingRecipient::from_bytes(&account.data).expect("Failed to deserialize recipient");

    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.recipient, *expected_recipient);
    assert_eq!(data.total_amount, expected_total_amount);
    assert_eq!(data.claimed_amount, expected_claimed_amount);
}

/// Assert that an account does not exist (was closed)
pub fn assert_account_closed(ctx: &TestContext, pubkey: &Pubkey) {
    assert!(ctx.get_account(pubkey).is_none(), "Account {} should be closed", pubkey);
}

/// Assert that an account exists
pub fn assert_account_exists(context: &TestContext, pubkey: &Pubkey) {
    let account = context.get_account(pubkey).unwrap_or_else(|| panic!("Account {pubkey} should exist"));
    assert!(!account.data.is_empty(), "Account data should not be empty");
}

/// Assert that an account does not exist
pub fn assert_account_not_exists(context: &TestContext, pubkey: &Pubkey) {
    assert!(context.get_account(pubkey).is_none(), "Account {pubkey} should not exist");
}
