use rewards_program_client::instructions::CloseMerkleDistributionBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::CreateMerkleDistributionSetup;
use crate::utils::{find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction};

pub struct CloseMerkleDistributionSetup {
    pub authority: Keypair,
    pub distribution_pda: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub authority_token_account: Pubkey,
    pub token_program: Pubkey,
    pub funded_amount: u64,
    pub clawback_ts: i64,
}

impl CloseMerkleDistributionSetup {
    pub fn builder(ctx: &mut TestContext) -> CloseMerkleDistributionSetupBuilder<'_> {
        CloseMerkleDistributionSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn from_distribution_setup(ctx: &mut TestContext, distribution_setup: &CreateMerkleDistributionSetup) -> Self {
        let instruction = distribution_setup.build_instruction(ctx);
        instruction.send_expect_success(ctx);

        let authority_token_account = if distribution_setup.token_program == TOKEN_2022_PROGRAM_ID {
            ctx.create_token_2022_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        } else {
            ctx.create_token_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        };

        Self {
            authority: distribution_setup.authority.insecure_clone(),
            distribution_pda: distribution_setup.distribution_pda,
            mint: distribution_setup.mint.pubkey(),
            vault: distribution_setup.vault,
            authority_token_account,
            token_program: distribution_setup.token_program,
            funded_amount: distribution_setup.amount,
            clawback_ts: distribution_setup.clawback_ts,
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseMerkleDistributionBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .distribution(self.distribution_pda)
            .mint(self.mint)
            .vault(self.vault)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "CloseMerkleDistribution",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        _ctx: &TestContext,
        wrong_authority: &Keypair,
        wrong_token_account: Pubkey,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CloseMerkleDistributionBuilder::new();
        builder
            .authority(wrong_authority.pubkey())
            .distribution(self.distribution_pda)
            .mint(self.mint)
            .vault(self.vault)
            .authority_token_account(wrong_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone()],
            name: "CloseMerkleDistribution",
        }
    }
}

pub struct CloseMerkleDistributionSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    warp_to_clawback: bool,
}

impl<'a> CloseMerkleDistributionSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID, amount: 1_000_000, warp_to_clawback: true }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn token_program(mut self, program: Pubkey) -> Self {
        self.token_program = program;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn warp_to_clawback(mut self, warp: bool) -> Self {
        self.warp_to_clawback = warp;
        self
    }

    pub fn build(self) -> CloseMerkleDistributionSetup {
        let current_ts = self.ctx.get_current_timestamp();
        let clawback_ts = current_ts + 86400; // 1 day from now

        let mut distribution_builder =
            CreateMerkleDistributionSetup::builder(self.ctx).amount(self.amount).clawback_ts(clawback_ts);

        if self.token_program == TOKEN_2022_PROGRAM_ID {
            distribution_builder = distribution_builder.token_2022();
        }

        let distribution_setup = distribution_builder.build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let authority_token_account = if self.token_program == TOKEN_2022_PROGRAM_ID {
            self.ctx
                .create_token_2022_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        } else {
            self.ctx.create_token_account(&distribution_setup.authority.pubkey(), &distribution_setup.mint.pubkey())
        };

        if self.warp_to_clawback {
            self.ctx.warp_to_timestamp(clawback_ts);
        }

        CloseMerkleDistributionSetup {
            authority: distribution_setup.authority,
            distribution_pda: distribution_setup.distribution_pda,
            mint: distribution_setup.mint.pubkey(),
            vault: distribution_setup.vault,
            authority_token_account,
            token_program: self.token_program,
            funded_amount: self.amount,
            clawback_ts,
        }
    }
}

pub struct CloseMerkleDistributionFixture;

impl InstructionTestFixture for CloseMerkleDistributionFixture {
    const INSTRUCTION_NAME: &'static str = "CloseMerkleDistribution";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CloseMerkleDistributionSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: authority
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// Account indices that must be writable:
    /// 0: authority
    /// 1: distribution
    /// 3: vault
    /// 4: authority_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 1, 3, 4]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(7)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
