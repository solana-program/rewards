use rewards_program_client::instructions::CloseMerkleClaimBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::{ClaimMerkleSetup, CloseMerkleDistributionSetup};
use crate::utils::{
    find_event_authority_pda, find_merkle_claim_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub struct CloseMerkleClaimSetup {
    pub claimant: Keypair,
    pub distribution_pda: Pubkey,
    pub claim_pda: Pubkey,
    pub token_program: Pubkey,
}

impl CloseMerkleClaimSetup {
    pub fn builder(ctx: &mut TestContext) -> CloseMerkleClaimSetupBuilder<'_> {
        CloseMerkleClaimSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseMerkleClaimBuilder::new();
        builder
            .claimant(self.claimant.pubkey())
            .distribution(self.distribution_pda)
            .claim_account(self.claim_pda)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.claimant.insecure_clone()],
            name: "CloseMerkleClaim",
        }
    }

    pub fn build_instruction_with_wrong_claimant(
        &self,
        _ctx: &TestContext,
        wrong_claimant: &Keypair,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let (wrong_claim_pda, _) = find_merkle_claim_pda(&self.distribution_pda, &wrong_claimant.pubkey());

        let mut builder = CloseMerkleClaimBuilder::new();
        builder
            .claimant(wrong_claimant.pubkey())
            .distribution(self.distribution_pda)
            .claim_account(wrong_claim_pda)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_claimant.insecure_clone()],
            name: "CloseMerkleClaim",
        }
    }
}

pub struct CloseMerkleClaimSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
}

impl<'a> CloseMerkleClaimSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn token_program(mut self, program: Pubkey) -> Self {
        self.token_program = program;
        self
    }

    pub fn build(self) -> CloseMerkleClaimSetup {
        // Create a claim setup first (which creates distribution and makes a claim)
        let mut claim_builder = ClaimMerkleSetup::builder(self.ctx);
        if self.token_program == TOKEN_2022_PROGRAM_ID {
            claim_builder = claim_builder.token_2022();
        }
        let claim_setup = claim_builder.build();

        // Execute a partial claim to create the claim account without auto-closing it
        let claim_ix = claim_setup.build_instruction_with_amount(self.ctx, claim_setup.total_amount / 2);
        claim_ix.send_expect_success(self.ctx);

        // Now close the distribution (need to warp to clawback time first)
        let current_ts = self.ctx.get_current_timestamp();
        // Warp forward to clawback time (distributions have default 1 year clawback)
        self.ctx.warp_to_timestamp(current_ts + 86400 * 365 + 1);

        let close_dist_setup = CloseMerkleDistributionSetup {
            authority: claim_setup.authority.insecure_clone(),
            distribution_pda: claim_setup.distribution_pda,
            mint: claim_setup.mint,
            distribution_vault: claim_setup.distribution_vault,
            authority_token_account: if self.token_program == TOKEN_2022_PROGRAM_ID {
                self.ctx.create_token_2022_account(&claim_setup.authority.pubkey(), &claim_setup.mint)
            } else {
                self.ctx.create_token_account(&claim_setup.authority.pubkey(), &claim_setup.mint)
            },
            token_program: self.token_program,
            funded_amount: 0,
            clawback_ts: current_ts + 86400 * 365,
        };

        let close_dist_ix = close_dist_setup.build_instruction(self.ctx);
        close_dist_ix.send_expect_success(self.ctx);

        CloseMerkleClaimSetup {
            claimant: claim_setup.claimant,
            distribution_pda: claim_setup.distribution_pda,
            claim_pda: claim_setup.claim_pda,
            token_program: self.token_program,
        }
    }
}

pub struct CloseMerkleClaimFixture;

impl InstructionTestFixture for CloseMerkleClaimFixture {
    const INSTRUCTION_NAME: &'static str = "CloseMerkleClaim";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CloseMerkleClaimSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: claimant
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// Account indices that must be writable:
    /// 0: claimant
    /// 2: claim_account
    fn required_writable() -> &'static [usize] {
        &[0, 2]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(4)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
