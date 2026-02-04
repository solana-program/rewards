//! Token2022 extension validation utilities.

use pinocchio::{account::AccountView, error::ProgramError};
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    state::Mint,
};

use crate::errors::RewardsProgramError;

/// Validates that a Token2022 mint does not have any dangerous extensions.
///
/// This function only checks mints owned by the Token-2022 program.
/// Regular SPL Token mints are allowed without extension checks.
///
/// # Blocked Extensions
/// - `PermanentDelegate`: Authority can transfer/burn tokens from ANY account
/// - `NonTransferable`: Tokens cannot be transferred
/// - `Pausable`: Authority can pause all transfers
///
/// # Arguments
/// * `mint` - The mint account to validate
///
/// # Returns
/// * `Ok(())` if the mint is safe to use
/// * `Err(RewardsProgramError::*)` if the mint has a blocked extension
#[inline(always)]
pub fn validate_mint_extensions(mint: &AccountView) -> Result<(), ProgramError> {
    if !mint.owned_by(&TOKEN_2022_PROGRAM_ID) {
        return Ok(());
    }

    let mint_data = mint.try_borrow()?;

    // Parse the mint with extensions
    let mint_state = StateWithExtensions::<Mint>::unpack(&mint_data)?;

    // Get all extension types present on this mint
    let extension_types = mint_state.get_extension_types()?;

    // Check each extension type against blocklist
    for ext_type in extension_types {
        match ext_type {
            ExtensionType::PermanentDelegate => {
                return Err(RewardsProgramError::PermanentDelegateNotAllowed.into());
            }
            ExtensionType::NonTransferable => {
                return Err(RewardsProgramError::NonTransferableNotAllowed.into());
            }
            ExtensionType::Pausable => {
                return Err(RewardsProgramError::PausableNotAllowed.into());
            }
            _ => {}
        }
    }

    Ok(())
}
