use solana_sdk::{instruction::InstructionError, pubkey::Pubkey};

use crate::utils::{assert_instruction_error, InstructionTestFixture, TestContext};

pub const RANDOM_PUBKEY: Pubkey = Pubkey::from_str_const("EpkG1ek8zrHWHqgUv42fTd6vJPsceSzkPSZfGaoLUGqf");

/// Test that removing a required signer fails with MissingRequiredSignature
pub fn test_missing_signer<T: InstructionTestFixture>(
    ctx: &mut TestContext,
    account_index: usize,
    signer_vec_index: usize,
) {
    let error = T::build_valid(ctx).without_signer(account_index, signer_vec_index).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::MissingRequiredSignature);
}

/// Test that making a required writable account read-only fails
pub fn test_not_writable<T: InstructionTestFixture>(ctx: &mut TestContext, account_index: usize) {
    let error = T::build_valid(ctx).with_readonly_at(account_index).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::Immutable);
}

/// Test that providing the wrong system program fails
pub fn test_wrong_system_program<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let index = T::system_program_index().expect("Instruction must have system_program_index");
    let error = T::build_valid(ctx).with_account_at(index, RANDOM_PUBKEY).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::IncorrectProgramId);
}

/// Test that providing the wrong current program fails
pub fn test_wrong_current_program<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let index = T::current_program_index().expect("Instruction must have current_program_index");
    let error = T::build_valid(ctx).with_account_at(index, RANDOM_PUBKEY).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::IncorrectProgramId);
}

/// Test that providing a wrong account at a given index fails
pub fn test_wrong_account<T: InstructionTestFixture>(
    ctx: &mut TestContext,
    account_index: usize,
    expected_error: InstructionError,
) {
    let error = T::build_valid(ctx).with_account_at(account_index, RANDOM_PUBKEY).send_expect_error(ctx);
    assert_instruction_error(error, expected_error);
}

/// Test that empty instruction data fails
pub fn test_empty_data<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let error = T::build_valid(ctx).with_data_len(0).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::InvalidInstructionData);
}

/// Test that truncated instruction data fails
pub fn test_truncated_data<T: InstructionTestFixture>(ctx: &mut TestContext) {
    let expected_len = T::data_len();
    if expected_len > 1 {
        let error = T::build_valid(ctx).with_data_len(expected_len - 1).send_expect_error(ctx);
        assert_instruction_error(error, InstructionError::InvalidInstructionData);
    }
}

/// Test that providing an invalid bump for a PDA fails
pub fn test_invalid_bump<T: InstructionTestFixture>(ctx: &mut TestContext, bump_byte_index: usize, invalid_bump: u8) {
    let error = T::build_valid(ctx).with_data_byte_at(bump_byte_index, invalid_bump).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::InvalidSeeds);
}

/// Test that providing an account with wrong ownership fails
pub fn test_wrong_owner<T: InstructionTestFixture>(ctx: &mut TestContext, account_index: usize) {
    let error = T::build_valid(ctx).with_account_at(account_index, RANDOM_PUBKEY).send_expect_error(ctx);
    assert_instruction_error(error, InstructionError::InvalidAccountOwner);
}
