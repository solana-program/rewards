use rewards_program_client::instructions::{
    ClaimContinuousBuilder, CloseRewardPoolBuilder, CreateRewardPoolBuilder, DistributeRewardBuilder, OptInBuilder,
    OptOutBuilder, SetBalanceBuilder, SyncBalanceBuilder,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::utils::{
    find_event_authority_pda, find_revocation_pda, find_reward_pool_pda, find_user_reward_account_pda,
    InstructionTestFixture, TestContext, TestInstruction,
};

pub const DEFAULT_TRACKED_BALANCE: u64 = 1_000_000;
pub const DEFAULT_REWARD_AMOUNT: u64 = 100_000;

pub struct CreateRewardPoolSetup {
    pub authority: Keypair,
    pub seed: Keypair,
    pub tracked_mint: Keypair,
    pub reward_mint: Keypair,
    pub reward_vault: Pubkey,
    pub reward_pool_pda: Pubkey,
    pub bump: u8,
    pub balance_source: u8,
    pub clawback_ts: i64,
    pub reward_token_program: Pubkey,
}

impl CreateRewardPoolSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_balance_source(ctx, 0)
    }

    pub fn new_authority_set(ctx: &mut TestContext) -> Self {
        Self::new_with_balance_source(ctx, 1)
    }

    fn new_with_balance_source(ctx: &mut TestContext, balance_source: u8) -> Self {
        let authority = ctx.create_funded_keypair();
        let seed = Keypair::new();
        let tracked_mint = Keypair::new();
        let reward_mint = Keypair::new();
        let reward_token_program = TOKEN_PROGRAM_ID;

        ctx.create_mint_for_program(&tracked_mint, &ctx.payer.pubkey(), 6, &TOKEN_PROGRAM_ID);
        ctx.create_mint_for_program(&reward_mint, &ctx.payer.pubkey(), 6, &reward_token_program);

        let (reward_pool_pda, bump) = find_reward_pool_pda(&reward_mint.pubkey(), &authority.pubkey(), &seed.pubkey());
        let reward_vault = ctx.create_ata_for_program(&reward_pool_pda, &reward_mint.pubkey(), &reward_token_program);

        CreateRewardPoolSetup {
            authority,
            seed,
            tracked_mint,
            reward_mint,
            reward_vault,
            reward_pool_pda,
            bump,
            balance_source,
            clawback_ts: 0,
            reward_token_program,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = CreateRewardPoolBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .seeds(self.seed.pubkey())
            .reward_pool(self.reward_pool_pda)
            .tracked_mint(self.tracked_mint.pubkey())
            .reward_mint(self.reward_mint.pubkey())
            .reward_vault(self.reward_vault)
            .reward_token_program(self.reward_token_program)
            .event_authority(event_authority)
            .bump(self.bump)
            .balance_source(self.balance_source)
            .clawback_ts(self.clawback_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone(), self.seed.insecure_clone()],
            name: "CreateRewardPool",
        }
    }
}

pub struct CreateRewardPoolFixture;

impl InstructionTestFixture for CreateRewardPoolFixture {
    const INSTRUCTION_NAME: &'static str = "CreateRewardPool";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CreateRewardPoolSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0, 1, 2] // payer, authority, seeds
    }

    fn required_writable() -> &'static [usize] {
        &[0, 3, 6] // payer, reward_pool, reward_vault
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 + 1 + 1 + 8 // discriminator + bump + balance_source + clawback_ts
    }
}

pub struct OptInSetup {
    pub pool_setup: CreateRewardPoolSetup,
    pub user: Keypair,
    pub user_tracked_token_account: Pubkey,
    pub user_reward_pda: Pubkey,
    pub user_reward_bump: u8,
    pub initial_balance: u64,
}

impl OptInSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_balance(ctx, DEFAULT_TRACKED_BALANCE, 0)
    }

    pub fn new_authority_set(ctx: &mut TestContext) -> Self {
        Self::new_authority_set_inner(ctx)
    }

    fn new_with_balance(ctx: &mut TestContext, balance: u64, balance_source: u8) -> Self {
        let pool_setup = if balance_source == 0 {
            CreateRewardPoolSetup::new(ctx)
        } else {
            CreateRewardPoolSetup::new_authority_set(ctx)
        };

        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let user = ctx.create_funded_keypair();
        let user_tracked_token_account =
            ctx.create_token_account_with_balance(&user.pubkey(), &pool_setup.tracked_mint.pubkey(), balance);

        let (user_reward_pda, user_reward_bump) =
            find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

        OptInSetup {
            pool_setup,
            user,
            user_tracked_token_account,
            user_reward_pda,
            user_reward_bump,
            initial_balance: balance,
        }
    }

    fn new_authority_set_inner(ctx: &mut TestContext) -> Self {
        Self::new_with_balance(ctx, 0, 1)
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let (revocation_pda, _) = find_revocation_pda(&self.pool_setup.reward_pool_pda, &self.user.pubkey());

        let mut builder = OptInBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .user(self.user.pubkey())
            .reward_pool(self.pool_setup.reward_pool_pda)
            .user_reward_account(self.user_reward_pda)
            .revocation_account(revocation_pda)
            .user_tracked_token_account(self.user_tracked_token_account)
            .tracked_mint(self.pool_setup.tracked_mint.pubkey())
            .tracked_token_program(TOKEN_PROGRAM_ID)
            .event_authority(event_authority)
            .bump(self.user_reward_bump);

        TestInstruction { instruction: builder.instruction(), signers: vec![self.user.insecure_clone()], name: "OptIn" }
    }
}

pub struct OptInFixture;

impl InstructionTestFixture for OptInFixture {
    const INSTRUCTION_NAME: &'static str = "OptIn";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = OptInSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0, 1] // payer, user
    }

    fn required_writable() -> &'static [usize] {
        &[0, 2, 3] // payer, reward_pool, user_reward_account
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(10)
    }

    fn data_len() -> usize {
        1 + 1 // discriminator + bump
    }
}

pub struct DistributeRewardSetup {
    pub opt_in_setup: OptInSetup,
    pub authority_token_account: Pubkey,
    pub amount: u64,
}

impl DistributeRewardSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_amount(ctx, DEFAULT_REWARD_AMOUNT)
    }

    pub fn new_with_amount(ctx: &mut TestContext, amount: u64) -> Self {
        let opt_in_setup = OptInSetup::new(ctx);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account = ctx.create_token_account_with_balance(
            &opt_in_setup.pool_setup.authority.pubkey(),
            &opt_in_setup.pool_setup.reward_mint.pubkey(),
            amount * 10,
        );

        DistributeRewardSetup { opt_in_setup, authority_token_account, amount }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let pool = &self.opt_in_setup.pool_setup;

        let mut builder = DistributeRewardBuilder::new();
        builder
            .authority(pool.authority.pubkey())
            .reward_pool(pool.reward_pool_pda)
            .reward_mint(pool.reward_mint.pubkey())
            .reward_vault(pool.reward_vault)
            .authority_token_account(self.authority_token_account)
            .reward_token_program(pool.reward_token_program)
            .event_authority(event_authority)
            .amount(self.amount);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![pool.authority.insecure_clone()],
            name: "DistributeReward",
        }
    }
}

pub struct DistributeRewardFixture;

impl InstructionTestFixture for DistributeRewardFixture {
    const INSTRUCTION_NAME: &'static str = "DistributeReward";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = DistributeRewardSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority
    }

    fn required_writable() -> &'static [usize] {
        &[1, 3, 4] // reward_pool, reward_vault, authority_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(7)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + amount
    }
}

pub fn build_claim_continuous_instruction(
    _ctx: &TestContext,
    pool_setup: &CreateRewardPoolSetup,
    user: &Keypair,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
    user_reward_token_account: &Pubkey,
    amount: u64,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = ClaimContinuousBuilder::new();
    builder
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user_tracked_token_account(*user_tracked_token_account)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(*user_reward_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .amount(amount);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![user.insecure_clone()],
        name: "ClaimContinuous",
    }
}

pub fn build_opt_out_instruction(
    _ctx: &TestContext,
    pool_setup: &CreateRewardPoolSetup,
    user: &Keypair,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
    user_reward_token_account: &Pubkey,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = OptOutBuilder::new();
    builder
        .user(user.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user_tracked_token_account(*user_tracked_token_account)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(*user_reward_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority);

    TestInstruction { instruction: builder.instruction(), signers: vec![user.insecure_clone()], name: "OptOut" }
}

pub fn build_sync_balance_instruction(
    pool_setup: &CreateRewardPoolSetup,
    user: &Pubkey,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
) -> TestInstruction {
    let mut builder = SyncBalanceBuilder::new();
    builder
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user(*user)
        .user_tracked_token_account(*user_tracked_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID);

    TestInstruction { instruction: builder.instruction(), signers: vec![], name: "SyncBalance" }
}

pub fn build_set_balance_instruction(
    pool_setup: &CreateRewardPoolSetup,
    user: &Pubkey,
    user_reward_pda: &Pubkey,
    balance: u64,
) -> TestInstruction {
    let mut builder = SetBalanceBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .user(*user)
        .balance(balance);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "SetBalance",
    }
}

pub fn build_close_reward_pool_instruction(
    _ctx: &TestContext,
    pool_setup: &CreateRewardPoolSetup,
    authority_token_account: &Pubkey,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();

    let mut builder = CloseRewardPoolBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .reward_mint(pool_setup.reward_mint.pubkey())
        .reward_vault(pool_setup.reward_vault)
        .authority_token_account(*authority_token_account)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "CloseRewardPool",
    }
}
