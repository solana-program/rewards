use pinocchio::{account::AccountView, error::ProgramError};

use crate::{
    traits::InstructionAccounts,
    utils::{
        validate_associated_token_account_address, validate_mint_extensions, verify_associated_token_program,
        verify_current_program, verify_event_authority, verify_owned_by, verify_readonly, verify_signer,
        verify_system_program, verify_token_program, verify_writable,
    },
};

pub struct CreateMerkleDistributionAccounts<'a> {
    pub payer: &'a AccountView,
    pub authority: &'a AccountView,
    pub seeds: &'a AccountView,
    pub distribution: &'a AccountView,
    pub mint: &'a AccountView,
    pub vault: &'a AccountView,
    pub authority_token_account: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
    pub associated_token_program: &'a AccountView,
    pub event_authority: &'a AccountView,
    pub program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for CreateMerkleDistributionAccounts<'a> {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [payer, authority, seeds, distribution, mint, vault, authority_token_account, system_program, token_program, associated_token_program, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. Validate signers
        verify_signer(payer, true)?;
        verify_signer(authority, false)?;
        verify_signer(seeds, false)?;

        // 2. Validate writable
        verify_writable(distribution, true)?;
        verify_writable(vault, true)?;
        verify_writable(authority_token_account, true)?;

        // 2b. Validate read-only accounts
        verify_readonly(mint)?;
        verify_readonly(seeds)?;

        // 3. Validate program IDs
        verify_system_program(system_program)?;
        verify_token_program(token_program)?;
        verify_associated_token_program(associated_token_program)?;
        verify_current_program(program)?;
        verify_event_authority(event_authority)?;

        // 4. (no accounts owned by current program for this instruction)

        // 5. Validate token account ownership and extensions
        verify_owned_by(mint, token_program.address())?;
        validate_mint_extensions(mint)?;
        verify_owned_by(authority_token_account, token_program.address())?;

        // 6. Validate ATA (vault may not be initialized yet, so just validate the address)
        validate_associated_token_account_address(vault, distribution.address(), mint, token_program)?;

        Ok(Self {
            payer,
            authority,
            seeds,
            distribution,
            mint,
            vault,
            authority_token_account,
            system_program,
            token_program,
            associated_token_program,
            event_authority,
            program,
        })
    }
}

impl<'a> InstructionAccounts<'a> for CreateMerkleDistributionAccounts<'a> {}
