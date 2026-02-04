use rewards_program_client::instructions::CreateVestingDistributionBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::ID as ASSOCIATED_TOKEN_PROGRAM_ID;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::utils::{
    find_event_authority_pda, find_vesting_distribution_pda, InstructionTestFixture, TestContext, TestInstruction,
    PROGRAM_ID,
};

pub const LINEAR_SCHEDULE: u8 = 0;
pub const DEFAULT_DISTRIBUTION_AMOUNT: u64 = 10_000_000;

pub struct CreateVestingDistributionSetup {
    pub authority: Keypair,
    pub seeds: Keypair,
    pub mint: Keypair,
    pub vault: Pubkey,
    pub authority_token_account: Pubkey,
    pub distribution_pda: Pubkey,
    pub bump: u8,
    pub amount: u64,
    pub token_program: Pubkey,
}

impl CreateVestingDistributionSetup {
    pub fn builder(ctx: &mut TestContext) -> CreateVestingDistributionSetupBuilder<'_> {
        CreateVestingDistributionSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CreateVestingDistributionBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .seeds(self.seeds.pubkey())
            .distribution(self.distribution_pda)
            .mint(self.mint.pubkey())
            .vault(self.vault)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .associated_token_program(ASSOCIATED_TOKEN_PROGRAM_ID)
            .event_authority(event_authority)
            .program(PROGRAM_ID)
            .bump(self.bump)
            .amount(self.amount);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.seeds.insecure_clone()],
            name: "CreateVestingDistribution",
        }
    }
}

pub struct CreateVestingDistributionSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
}

impl<'a> CreateVestingDistributionSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID, amount: DEFAULT_DISTRIBUTION_AMOUNT }
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

    pub fn build(self) -> CreateVestingDistributionSetup {
        let authority = self.ctx.create_funded_keypair();
        let seeds = Keypair::new();
        let mint = Keypair::new();
        let token_program = self.token_program;

        if token_program == TOKEN_2022_PROGRAM_ID {
            self.ctx.create_token_2022_mint(&mint, &self.ctx.payer.pubkey(), 6);
        } else {
            self.ctx.create_mint(&mint, &self.ctx.payer.pubkey(), 6);
        }

        let (distribution_pda, bump) =
            find_vesting_distribution_pda(&mint.pubkey(), &authority.pubkey(), &seeds.pubkey());

        let vault = if token_program == TOKEN_2022_PROGRAM_ID {
            self.ctx.create_token_2022_account(&distribution_pda, &mint.pubkey())
        } else {
            self.ctx.create_token_account(&distribution_pda, &mint.pubkey())
        };

        let authority_token_account = if token_program == TOKEN_2022_PROGRAM_ID {
            self.ctx.create_token_2022_account_with_balance(&authority.pubkey(), &mint.pubkey(), self.amount)
        } else {
            self.ctx.create_token_account_with_balance(&authority.pubkey(), &mint.pubkey(), self.amount)
        };

        CreateVestingDistributionSetup {
            authority,
            seeds,
            mint,
            vault,
            authority_token_account,
            distribution_pda,
            bump,
            amount: self.amount,
            token_program,
        }
    }
}

pub struct CreateVestingDistributionFixture;

impl InstructionTestFixture for CreateVestingDistributionFixture {
    const INSTRUCTION_NAME: &'static str = "CreateVestingDistribution";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CreateVestingDistributionSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: payer (handled by TestContext)
    /// 1: authority
    /// 2: seeds
    fn required_signers() -> &'static [usize] {
        &[0, 1, 2]
    }

    /// Account indices that must be writable:
    /// 0: payer (handled by TestContext)
    /// 3: distribution
    /// 5: vault
    /// 6: authority_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 3, 5, 6]
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 + 1 + 8 // discriminator + bump + amount
    }
}
