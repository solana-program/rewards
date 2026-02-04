use codama::CodamaInstructions;

/// Instructions for the Rewards Program.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum RewardsProgramInstruction {
    /// Create a new vesting distribution with initial funding.
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "seeds", signer))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "authority_token_account", writable))]
    #[codama(account(name = "system_program"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "associated_token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "program"))]
    CreateVestingDistribution {
        /// Bump for the distribution PDA
        bump: u8,
        /// Amount of tokens to lock in vault (must be > 0)
        amount: u64,
    } = 0,

    /// Add a recipient to a vesting distribution.
    /// Each recipient has their own vesting schedule.
    /// Validates that total_allocated + amount does not exceed vault balance.
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "authority", signer))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "recipient_account", writable))]
    #[codama(account(name = "recipient"))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault"))]
    #[codama(account(name = "system_program"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "program"))]
    AddVestingRecipient {
        /// Bump for the recipient PDA
        bump: u8,
        /// Amount allocated to the recipient
        amount: u64,
        /// Schedule type (0 = linear)
        schedule_type: u8,
        /// Vesting start timestamp
        start_ts: i64,
        /// Vesting end timestamp
        end_ts: i64,
    } = 1,

    /// Claim vested tokens.
    #[codama(account(name = "recipient", signer))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "recipient_account", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "recipient_token_account", writable))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "program"))]
    ClaimVesting {
        /// Amount to claim. 0 = claim all available.
        amount: u64,
    } = 2,

    /// Close a vesting distribution and recover remaining tokens.
    #[codama(account(name = "authority", signer, writable))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "authority_token_account", writable))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "program"))]
    CloseVestingDistribution {} = 3,

    /// Emit event data via CPI (prevents log truncation).
    #[codama(account(name = "event_authority", signer))]
    EmitEvent {} = 4,
}
