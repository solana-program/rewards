use rewards_program_client::instructions::{
    ClaimContinuousBuilder, CloseRewardPoolBuilder, CreateRewardPoolBuilder, DistributeRewardBuilder, OptInBuilder,
    OptOutBuilder, RevokeUserBuilder, SetBalanceBuilder, SyncBalanceBuilder,
};
use rewards_program_client::types::RevokeMode;
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
    pub revocable: u8,
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
            revocable: 0,
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
            .revocable(self.revocable)
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
        1 + 1 + 1 + 1 + 8 // discriminator + bump + balance_source + revocable + clawback_ts
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

    pub fn new_from_pool(ctx: &mut TestContext, pool_setup: CreateRewardPoolSetup) -> Self {
        let user = ctx.create_funded_keypair();
        let user_tracked_token_account = ctx.create_token_account_with_balance(
            &user.pubkey(),
            &pool_setup.tracked_mint.pubkey(),
            DEFAULT_TRACKED_BALANCE,
        );

        let (user_reward_pda, user_reward_bump) =
            find_user_reward_account_pda(&pool_setup.reward_pool_pda, &user.pubkey());

        OptInSetup {
            pool_setup,
            user,
            user_tracked_token_account,
            user_reward_pda,
            user_reward_bump,
            initial_balance: DEFAULT_TRACKED_BALANCE,
        }
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

pub struct ClaimContinuousSetup {
    pub distribute_setup: DistributeRewardSetup,
    pub user_reward_token_account: Pubkey,
}

impl ClaimContinuousSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let distribute_setup = DistributeRewardSetup::new(ctx);
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

        ClaimContinuousSetup { distribute_setup, user_reward_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        let user_reward_pda = &self.distribute_setup.opt_in_setup.user_reward_pda;
        let user_tracked_ta = &self.distribute_setup.opt_in_setup.user_tracked_token_account;

        build_claim_continuous_instruction(
            ctx,
            pool_setup,
            user,
            user_reward_pda,
            user_tracked_ta,
            &self.user_reward_token_account,
            0,
        )
    }
}

pub struct ClaimContinuousFixture;

impl InstructionTestFixture for ClaimContinuousFixture {
    const INSTRUCTION_NAME: &'static str = "ClaimContinuous";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClaimContinuousSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // user
    }

    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 5] // reward_pool, user_reward_account, reward_vault, user_reward_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + amount
    }
}

pub struct OptOutSetup {
    pub distribute_setup: DistributeRewardSetup,
    pub user_reward_token_account: Pubkey,
}

impl OptOutSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let distribute_setup = DistributeRewardSetup::new(ctx);
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());

        OptOutSetup { distribute_setup, user_reward_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        let user_reward_pda = &self.distribute_setup.opt_in_setup.user_reward_pda;
        let user_tracked_ta = &self.distribute_setup.opt_in_setup.user_tracked_token_account;

        build_opt_out_instruction(
            ctx,
            pool_setup,
            user,
            user_reward_pda,
            user_tracked_ta,
            &self.user_reward_token_account,
        )
    }
}

pub struct OptOutFixture;

impl InstructionTestFixture for OptOutFixture {
    const INSTRUCTION_NAME: &'static str = "OptOut";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = OptOutSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // user
    }

    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 5] // reward_pool, user_reward_account, reward_vault, user_reward_token_account
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        1 // discriminator only (no data)
    }
}

pub struct SyncBalanceSetup {
    pub opt_in_setup: OptInSetup,
}

impl SyncBalanceSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let opt_in_setup = OptInSetup::new(ctx);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);
        SyncBalanceSetup { opt_in_setup }
    }

    pub fn build_instruction(&self) -> TestInstruction {
        build_sync_balance_instruction(
            &self.opt_in_setup.pool_setup,
            &self.opt_in_setup.user.pubkey(),
            &self.opt_in_setup.user_reward_pda,
            &self.opt_in_setup.user_tracked_token_account,
        )
    }
}

pub struct SyncBalanceFixture;

impl InstructionTestFixture for SyncBalanceFixture {
    const INSTRUCTION_NAME: &'static str = "SyncBalance";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = SyncBalanceSetup::new(ctx);
        setup.build_instruction()
    }

    fn required_signers() -> &'static [usize] {
        &[] // permissionless
    }

    fn required_writable() -> &'static [usize] {
        &[0, 1] // reward_pool, user_reward_account
    }

    fn current_program_index() -> Option<usize> {
        None
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}

pub struct SetBalanceSetup {
    pub opt_in_setup: OptInSetup,
}

impl SetBalanceSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let opt_in_setup = OptInSetup::new_authority_set(ctx);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);
        SetBalanceSetup { opt_in_setup }
    }

    pub fn build_instruction(&self) -> TestInstruction {
        build_set_balance_instruction(
            &self.opt_in_setup.pool_setup,
            &self.opt_in_setup.user.pubkey(),
            &self.opt_in_setup.user_reward_pda,
            500_000,
        )
    }
}

pub struct SetBalanceFixture;

impl InstructionTestFixture for SetBalanceFixture {
    const INSTRUCTION_NAME: &'static str = "SetBalance";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = SetBalanceSetup::new(ctx);
        setup.build_instruction()
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority
    }

    fn required_writable() -> &'static [usize] {
        &[1, 2] // reward_pool, user_reward_account
    }

    fn current_program_index() -> Option<usize> {
        None
    }

    fn data_len() -> usize {
        1 + 8 // discriminator + balance
    }
}

pub struct CloseRewardPoolSetup {
    pub pool_setup: CreateRewardPoolSetup,
    pub authority_token_account: Pubkey,
}

impl CloseRewardPoolSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        let pool_setup = CreateRewardPoolSetup::new(ctx);
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account =
            ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

        CloseRewardPoolSetup { pool_setup, authority_token_account }
    }

    pub fn new_with_clawback(ctx: &mut TestContext, clawback_ts: i64) -> Self {
        let mut pool_setup = CreateRewardPoolSetup::new(ctx);
        pool_setup.clawback_ts = clawback_ts;
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account =
            ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

        CloseRewardPoolSetup { pool_setup, authority_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        build_close_reward_pool_instruction(ctx, &self.pool_setup, &self.authority_token_account)
    }
}

pub struct CloseRewardPoolFixture;

impl InstructionTestFixture for CloseRewardPoolFixture {
    const INSTRUCTION_NAME: &'static str = "CloseRewardPool";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = CloseRewardPoolSetup::new(ctx);
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
        1 // discriminator only
    }
}

pub struct RevokeUserSetup {
    pub distribute_setup: DistributeRewardSetup,
    pub user_reward_token_account: Pubkey,
    pub authority_reward_token_account: Pubkey,
}

impl RevokeUserSetup {
    pub fn new(ctx: &mut TestContext) -> Self {
        Self::new_with_revocable(ctx, 3)
    }

    pub fn new_with_revocable(ctx: &mut TestContext, revocable: u8) -> Self {
        let mut pool_setup = CreateRewardPoolSetup::new(ctx);
        pool_setup.revocable = revocable;
        pool_setup.build_instruction(ctx).send_expect_success(ctx);

        let opt_in_setup = OptInSetup::new_from_pool(ctx, pool_setup);
        opt_in_setup.build_instruction(ctx).send_expect_success(ctx);

        let authority_token_account = ctx.create_token_account_with_balance(
            &opt_in_setup.pool_setup.authority.pubkey(),
            &opt_in_setup.pool_setup.reward_mint.pubkey(),
            DEFAULT_REWARD_AMOUNT * 10,
        );

        let distribute_setup =
            DistributeRewardSetup { opt_in_setup, authority_token_account, amount: DEFAULT_REWARD_AMOUNT };
        distribute_setup.build_instruction(ctx).send_expect_success(ctx);

        let pool_setup = &distribute_setup.opt_in_setup.pool_setup;
        let user = &distribute_setup.opt_in_setup.user;
        let user_reward_token_account = ctx.create_token_account(&user.pubkey(), &pool_setup.reward_mint.pubkey());
        let authority_reward_token_account =
            ctx.create_token_account(&pool_setup.authority.pubkey(), &pool_setup.reward_mint.pubkey());

        RevokeUserSetup { distribute_setup, user_reward_token_account, authority_reward_token_account }
    }

    pub fn build_instruction(&self, ctx: &TestContext, revoke_mode: RevokeMode) -> TestInstruction {
        let pool_setup = &self.distribute_setup.opt_in_setup.pool_setup;
        let user = &self.distribute_setup.opt_in_setup.user;
        let user_reward_pda = &self.distribute_setup.opt_in_setup.user_reward_pda;
        let user_tracked_ta = &self.distribute_setup.opt_in_setup.user_tracked_token_account;

        build_revoke_user_instruction(
            ctx,
            pool_setup,
            user,
            user_reward_pda,
            user_tracked_ta,
            &self.user_reward_token_account,
            &self.authority_reward_token_account,
            revoke_mode,
        )
    }
}

pub struct RevokeUserFixture;

impl InstructionTestFixture for RevokeUserFixture {
    const INSTRUCTION_NAME: &'static str = "RevokeUser";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = RevokeUserSetup::new(ctx);
        setup.build_instruction(ctx, RevokeMode::NonVested)
    }

    fn required_signers() -> &'static [usize] {
        &[0] // authority (payer is authority, auto-signed via signers vec)
    }

    fn required_writable() -> &'static [usize] {
        &[2, 3, 4, 5, 7, 8, 9] // reward_pool, user_reward_account, revocation_account, user, reward_vault, user_reward_token_account, authority_reward_token_account
    }

    fn system_program_index() -> Option<usize> {
        Some(12)
    }

    fn current_program_index() -> Option<usize> {
        Some(16)
    }

    fn data_len() -> usize {
        1 + 1 // discriminator + revoke_mode
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_revoke_user_instruction(
    ctx: &TestContext,
    pool_setup: &CreateRewardPoolSetup,
    user: &Keypair,
    user_reward_pda: &Pubkey,
    user_tracked_token_account: &Pubkey,
    user_reward_token_account: &Pubkey,
    authority_reward_token_account: &Pubkey,
    revoke_mode: RevokeMode,
) -> TestInstruction {
    let (event_authority, _) = find_event_authority_pda();
    let (revocation_pda, _) = find_revocation_pda(&pool_setup.reward_pool_pda, &user.pubkey());

    let mut builder = RevokeUserBuilder::new();
    builder
        .authority(pool_setup.authority.pubkey())
        .payer(ctx.payer.pubkey())
        .reward_pool(pool_setup.reward_pool_pda)
        .user_reward_account(*user_reward_pda)
        .revocation_account(revocation_pda)
        .user(user.pubkey())
        .user_tracked_token_account(*user_tracked_token_account)
        .reward_vault(pool_setup.reward_vault)
        .user_reward_token_account(*user_reward_token_account)
        .authority_reward_token_account(*authority_reward_token_account)
        .tracked_mint(pool_setup.tracked_mint.pubkey())
        .reward_mint(pool_setup.reward_mint.pubkey())
        .tracked_token_program(TOKEN_PROGRAM_ID)
        .reward_token_program(pool_setup.reward_token_program)
        .event_authority(event_authority)
        .revoke_mode(revoke_mode);

    TestInstruction {
        instruction: builder.instruction(),
        signers: vec![pool_setup.authority.insecure_clone()],
        name: "RevokeUser",
    }
}
