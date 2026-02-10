use rewards_program_client::instructions::{AddDirectRecipientBuilder, RevokeDirectRecipientBuilder};
use rewards_program_client::types::{RevokeMode, VestingSchedule};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::CreateDirectDistributionSetup;
use crate::utils::{
    find_direct_recipient_pda, find_event_authority_pda, InstructionTestFixture, TestContext, TestInstruction,
};

pub const DEFAULT_REVOKE_AMOUNT: u64 = 1_000_000;

pub struct RevokeDirectRecipientSetup {
    pub authority: Keypair,
    pub distribution_pda: Pubkey,
    pub recipient: Keypair,
    pub recipient_pda: Pubkey,
    pub payer: Keypair,
    pub mint: Pubkey,
    pub distribution_vault: Pubkey,
    pub recipient_token_account: Pubkey,
    pub token_program: Pubkey,
    pub amount: u64,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl RevokeDirectRecipientSetup {
    pub fn builder(ctx: &mut TestContext) -> RevokeDirectRecipientSetupBuilder<'_> {
        RevokeDirectRecipientSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn build_instruction(&self, _ctx: &TestContext, revoke_mode: RevokeMode) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = RevokeDirectRecipientBuilder::new();
        builder
            .authority(self.authority.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .recipient(self.recipient.pubkey())
            .payer(self.payer.pubkey())
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .recipient_token_account(self.recipient_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .revoke_mode(revoke_mode);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "RevokeDirectRecipient",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        _ctx: &TestContext,
        wrong_authority: &Keypair,
        revoke_mode: RevokeMode,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = RevokeDirectRecipientBuilder::new();
        builder
            .authority(wrong_authority.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .recipient(self.recipient.pubkey())
            .payer(self.payer.pubkey())
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .recipient_token_account(self.recipient_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .revoke_mode(revoke_mode);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone()],
            name: "RevokeDirectRecipient",
        }
    }
}

pub struct RevokeDirectRecipientSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    schedule: Option<VestingSchedule>,
}

impl<'a> RevokeDirectRecipientSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID, amount: DEFAULT_REVOKE_AMOUNT, schedule: None }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn schedule(mut self, schedule: VestingSchedule) -> Self {
        self.schedule = Some(schedule);
        self
    }

    pub fn build(self) -> RevokeDirectRecipientSetup {
        let distribution_setup = CreateDirectDistributionSetup::builder(self.ctx)
            .token_program(self.token_program)
            .revocable(1)
            .build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let current_ts = self.ctx.get_current_timestamp();
        let schedule = self
            .schedule
            .unwrap_or(VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 });

        let (start_ts, end_ts) = match &schedule {
            VestingSchedule::Linear { start_ts, end_ts } => (*start_ts, *end_ts),
            VestingSchedule::CliffLinear { start_ts, end_ts, .. } => (*start_ts, *end_ts),
            VestingSchedule::Cliff { cliff_ts } => (0, *cliff_ts),
            VestingSchedule::Immediate => (0, 0),
        };

        let recipient = self.ctx.create_funded_keypair();
        let rent_payer = self.ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let authority_token_account = self.ctx.create_ata_for_program_with_balance(
            &distribution_setup.authority.pubkey(),
            &distribution_setup.mint.pubkey(),
            self.amount,
            &self.token_program,
        );

        let (event_authority, _) = find_event_authority_pda();
        let mut add_builder = AddDirectRecipientBuilder::new();
        add_builder
            .payer(rent_payer.pubkey())
            .authority(distribution_setup.authority.pubkey())
            .distribution(distribution_setup.distribution_pda)
            .recipient_account(recipient_pda)
            .recipient(recipient.pubkey())
            .mint(distribution_setup.mint.pubkey())
            .distribution_vault(distribution_setup.distribution_vault)
            .authority_token_account(authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(recipient_bump)
            .amount(self.amount)
            .schedule(schedule);

        let add_ix = TestInstruction {
            instruction: add_builder.instruction(),
            signers: vec![rent_payer.insecure_clone(), distribution_setup.authority.insecure_clone()],
            name: "AddDirectRecipient",
        };
        add_ix.send_expect_success(self.ctx);

        let recipient_token_account = self.ctx.create_ata_for_program(
            &recipient.pubkey(),
            &distribution_setup.mint.pubkey(),
            &self.token_program,
        );

        RevokeDirectRecipientSetup {
            authority: distribution_setup.authority,
            distribution_pda: distribution_setup.distribution_pda,
            recipient,
            recipient_pda,
            payer: rent_payer,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            recipient_token_account,
            token_program: self.token_program,
            amount: self.amount,
            start_ts,
            end_ts,
        }
    }
}

pub struct RevokeDirectRecipientFixture;

impl InstructionTestFixture for RevokeDirectRecipientFixture {
    const INSTRUCTION_NAME: &'static str = "RevokeDirectRecipient";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = RevokeDirectRecipientSetup::new(ctx);
        setup.build_instruction(ctx, RevokeMode::NonVested)
    }

    /// Account indices that must be signers:
    /// 0: authority
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// Account indices that must be writable:
    /// 1: distribution
    /// 2: recipient_account
    /// 4: payer
    /// 6: vault
    /// 7: recipient_token_account
    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 6, 7]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(10)
    }

    fn data_len() -> usize {
        1 + 1 // discriminator + revoke_mode
    }
}
