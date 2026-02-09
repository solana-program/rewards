use pinocchio::{account::AccountView, error::ProgramError, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{errors::RewardsProgramError, traits::DistributionSigner};

use super::get_mint_decimals;

/// Context for claim transfer operations.
///
/// Groups the accounts needed for transferring tokens from distribution_vault to recipient.
pub struct ClaimTransferContext<'a> {
    pub distribution_vault: &'a AccountView,
    pub mint: &'a AccountView,
    pub destination: &'a AccountView,
    pub distribution_account: &'a AccountView,
    pub token_program: &'a Address,
}

/// Resolves the actual claim amount based on request and available balance.
///
/// # Arguments
/// * `requested` - Amount requested (0 = claim all available)
/// * `claimable` - Maximum amount available to claim
///
/// # Returns
/// * `Ok(amount)` - The resolved claim amount
/// * `Err` - If requested exceeds claimable
#[inline(always)]
pub fn resolve_claim_amount(requested: u64, claimable: u64) -> Result<u64, ProgramError> {
    if claimable == 0 {
        return Err(RewardsProgramError::NothingToClaim.into());
    }

    if requested == 0 {
        Ok(claimable)
    } else if requested > claimable {
        Err(RewardsProgramError::ExceedsClaimableAmount.into())
    } else {
        Ok(requested)
    }
}

/// Transfers tokens from the distribution vault to recipient using the distribution as signer.
///
/// # Arguments
/// * `distribution` - The distribution that owns the vault (implements DistributionSigner)
/// * `ctx` - Transfer context containing all required accounts
/// * `amount` - Amount to transfer
#[inline(always)]
pub fn transfer_from_distribution_vault<D: DistributionSigner>(
    distribution: &D,
    ctx: &ClaimTransferContext,
    amount: u64,
) -> ProgramResult {
    let decimals = get_mint_decimals(ctx.mint)?;

    distribution.with_signer(|signers| {
        TransferChecked {
            from: ctx.distribution_vault,
            mint: ctx.mint,
            to: ctx.destination,
            authority: ctx.distribution_account,
            amount,
            decimals,
            token_program: ctx.token_program,
        }
        .invoke_signed(signers)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_claim_amount_claim_all() {
        let result = resolve_claim_amount(0, 1000);
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_resolve_claim_amount_partial() {
        let result = resolve_claim_amount(500, 1000);
        assert_eq!(result.unwrap(), 500);
    }

    #[test]
    fn test_resolve_claim_amount_exact() {
        let result = resolve_claim_amount(1000, 1000);
        assert_eq!(result.unwrap(), 1000);
    }

    #[test]
    fn test_resolve_claim_amount_exceeds() {
        let result = resolve_claim_amount(1500, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_claim_amount_nothing_to_claim() {
        let result = resolve_claim_amount(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_claim_amount_nothing_to_claim_with_request() {
        let result = resolve_claim_amount(500, 0);
        assert!(result.is_err());
    }
}
