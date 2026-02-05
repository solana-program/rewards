use alloc::vec::Vec;
use codama::CodamaInstructions;

/// Instructions for the Rewards Program.
#[repr(C, u8)]
#[derive(Clone, Debug, PartialEq, CodamaInstructions)]
pub enum RewardsProgramInstruction {
    /// Create a new direct distribution with initial funding.
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
    #[codama(account(name = "rewardsProgram"))]
    CreateDirectDistribution {
        /// Bump for the distribution PDA
        bump: u8,
        /// Amount of tokens to lock in vault (must be > 0)
        amount: u64,
    } = 0,

    /// Add a recipient to a direct distribution.
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
    #[codama(account(name = "rewardsProgram"))]
    AddDirectRecipient {
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

    /// Claim tokens from a direct distribution.
    #[codama(account(name = "recipient", signer))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "recipient_account", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "recipient_token_account", writable))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "rewardsProgram"))]
    ClaimDirect {
        /// Amount to claim. 0 = claim all available.
        amount: u64,
    } = 2,

    /// Close a direct distribution and recover remaining tokens.
    #[codama(account(name = "authority", signer, writable))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "authority_token_account", writable))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "rewardsProgram"))]
    CloseDirectDistribution {} = 3,

    /// Close a direct recipient account after fully claiming, recovering rent.
    #[codama(account(name = "recipient", signer))]
    #[codama(account(name = "payer", writable))]
    #[codama(account(name = "distribution"))]
    #[codama(account(name = "recipient_account", writable))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "rewardsProgram"))]
    CloseDirectRecipient {} = 4,

    /// Create a new merkle distribution with initial funding.
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
    #[codama(account(name = "rewardsProgram"))]
    CreateMerkleDistribution {
        /// Bump for the distribution PDA
        bump: u8,
        /// Amount of tokens to deposit in vault
        amount: u64,
        /// Merkle root hash
        merkle_root: [u8; 32],
        /// Total amount claimable by all recipients
        total_amount: u64,
        /// Timestamp after which authority can close the distribution
        clawback_ts: i64,
    } = 5,

    /// Claim tokens from a merkle distribution.
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "claimant", signer))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "claim_account", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "claimant_token_account", writable))]
    #[codama(account(name = "system_program"))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "rewardsProgram"))]
    ClaimMerkle {
        /// Bump for the claim PDA
        claim_bump: u8,
        /// Total amount allocated to claimant (from merkle leaf)
        total_amount: u64,
        /// Vesting schedule type (from merkle leaf): 0 = Immediate, 1 = Linear
        schedule_type: u8,
        /// Vesting start timestamp (from merkle leaf)
        start_ts: i64,
        /// Vesting end timestamp (from merkle leaf)
        end_ts: i64,
        /// Amount to claim (0 = claim all available)
        amount: u64,
        /// Merkle proof
        proof: Vec<[u8; 32]>,
    } = 6,

    /// Close a merkle claim after distribution is closed.
    #[codama(account(name = "claimant", signer, writable))]
    #[codama(account(name = "distribution"))]
    #[codama(account(name = "claim_account", writable))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "rewardsProgram"))]
    CloseMerkleClaim {} = 7,

    /// Close a merkle distribution after clawback timestamp.
    #[codama(account(name = "authority", signer, writable))]
    #[codama(account(name = "distribution", writable))]
    #[codama(account(name = "mint"))]
    #[codama(account(name = "vault", writable))]
    #[codama(account(name = "authority_token_account", writable))]
    #[codama(account(name = "token_program"))]
    #[codama(account(name = "event_authority"))]
    #[codama(account(name = "rewardsProgram"))]
    CloseMerkleDistribution {} = 8,

    /// Emit event data via CPI (prevents log truncation).
    #[codama(account(name = "event_authority", signer))]
    EmitEvent {} = 228,
}
