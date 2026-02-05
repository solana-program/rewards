use rewards_program_client::instructions::CreateMerkleDistributionBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::utils::{
    find_event_authority_pda, find_merkle_distribution_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub const IMMEDIATE_SCHEDULE: u8 = 0;
pub const LINEAR_SCHEDULE: u8 = 1;
pub const DEFAULT_MERKLE_DISTRIBUTION_AMOUNT: u64 = 10_000_000;
pub const DEFAULT_CLAWBACK_OFFSET: i64 = 86400 * 365; // 1 year

pub struct CreateMerkleDistributionSetup {
    pub authority: Keypair,
    pub seeds: Keypair,
    pub mint: Keypair,
    pub vault: Pubkey,
    pub authority_token_account: Pubkey,
    pub distribution_pda: Pubkey,
    pub bump: u8,
    pub amount: u64,
    pub total_amount: u64,
    pub merkle_root: [u8; 32],
    pub clawback_ts: i64,
    pub token_program: Pubkey,
}

impl CreateMerkleDistributionSetup {
    pub fn builder(ctx: &mut TestContext) -> CreateMerkleDistributionSetupBuilder<'_> {
        CreateMerkleDistributionSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CreateMerkleDistributionBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .seeds(self.seeds.pubkey())
            .distribution(self.distribution_pda)
            .mint(self.mint.pubkey())
            .vault(self.vault)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(self.bump)
            .amount(self.amount)
            .merkle_root(self.merkle_root)
            .total_amount(self.total_amount)
            .clawback_ts(self.clawback_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.seeds.insecure_clone()],
            name: "CreateMerkleDistribution",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        ctx: &TestContext,
        wrong_authority: &Keypair,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CreateMerkleDistributionBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(wrong_authority.pubkey())
            .seeds(self.seeds.pubkey())
            .distribution(self.distribution_pda)
            .mint(self.mint.pubkey())
            .vault(self.vault)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(self.bump)
            .amount(self.amount)
            .merkle_root(self.merkle_root)
            .total_amount(self.total_amount)
            .clawback_ts(self.clawback_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone(), self.seeds.insecure_clone()],
            name: "CreateMerkleDistribution",
        }
    }
}

pub struct CreateMerkleDistributionSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    total_amount: Option<u64>,
    merkle_root: Option<[u8; 32]>,
    clawback_ts: Option<i64>,
}

impl<'a> CreateMerkleDistributionSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self {
            ctx,
            token_program: TOKEN_PROGRAM_ID,
            amount: DEFAULT_MERKLE_DISTRIBUTION_AMOUNT,
            total_amount: None,
            merkle_root: None,
            clawback_ts: None,
        }
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

    pub fn total_amount(mut self, total_amount: u64) -> Self {
        self.total_amount = Some(total_amount);
        self
    }

    pub fn merkle_root(mut self, merkle_root: [u8; 32]) -> Self {
        self.merkle_root = Some(merkle_root);
        self
    }

    pub fn clawback_ts(mut self, clawback_ts: i64) -> Self {
        self.clawback_ts = Some(clawback_ts);
        self
    }

    pub fn build(self) -> CreateMerkleDistributionSetup {
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
            find_merkle_distribution_pda(&mint.pubkey(), &authority.pubkey(), &seeds.pubkey());

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

        let current_ts = self.ctx.get_current_timestamp();
        let clawback_ts = self.clawback_ts.unwrap_or(current_ts + DEFAULT_CLAWBACK_OFFSET);
        let total_amount = self.total_amount.unwrap_or(self.amount);
        let merkle_root = self.merkle_root.unwrap_or([1u8; 32]); // Default placeholder root

        CreateMerkleDistributionSetup {
            authority,
            seeds,
            mint,
            vault,
            authority_token_account,
            distribution_pda,
            bump,
            amount: self.amount,
            total_amount,
            merkle_root,
            clawback_ts,
            token_program,
        }
    }
}

pub struct CreateMerkleDistributionFixture;

impl InstructionTestFixture for CreateMerkleDistributionFixture {
    const INSTRUCTION_NAME: &'static str = "CreateMerkleDistribution";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CreateMerkleDistributionSetup::new(ctx);
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
        1 + 1 + 8 + 32 + 8 + 8 // discriminator + bump + amount + merkle_root + total_amount + clawback_ts
    }
}
