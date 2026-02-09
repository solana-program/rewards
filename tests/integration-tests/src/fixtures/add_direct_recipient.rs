use rewards_program_client::{instructions::AddDirectRecipientBuilder, types::VestingSchedule};
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

pub const DEFAULT_RECIPIENT_AMOUNT: u64 = 1_000_000;

pub struct AddDirectRecipientSetup {
    pub authority: Keypair,
    pub distribution_pda: Pubkey,
    pub recipient: Keypair,
    pub recipient_pda: Pubkey,
    pub recipient_bump: u8,
    pub amount: u64,
    pub schedule: VestingSchedule,
    pub token_program: Pubkey,
    pub mint: Pubkey,
    pub distribution_vault: Pubkey,
    pub authority_token_account: Pubkey,
}

impl AddDirectRecipientSetup {
    pub fn builder(ctx: &mut TestContext) -> AddDirectRecipientSetupBuilder<'_> {
        AddDirectRecipientSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn from_distribution_setup(ctx: &mut TestContext, distribution_setup: &CreateDirectDistributionSetup) -> Self {
        let instruction = distribution_setup.build_instruction(ctx);
        instruction.send_expect_success(ctx);

        let recipient = ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let current_ts = ctx.get_current_timestamp();

        let authority_token_account = ctx.create_ata_for_program_with_balance(
            &distribution_setup.authority.pubkey(),
            &distribution_setup.mint.pubkey(),
            DEFAULT_RECIPIENT_AMOUNT,
            &distribution_setup.token_program,
        );

        Self {
            authority: distribution_setup.authority.insecure_clone(),
            distribution_pda: distribution_setup.distribution_pda,
            recipient,
            recipient_pda,
            recipient_bump,
            amount: DEFAULT_RECIPIENT_AMOUNT,
            schedule: VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 },
            token_program: distribution_setup.token_program,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            authority_token_account,
        }
    }

    pub fn start_ts(&self) -> i64 {
        match &self.schedule {
            VestingSchedule::Immediate => 0,
            VestingSchedule::Linear { start_ts, .. } => *start_ts,
            VestingSchedule::Cliff { .. } => 0,
            VestingSchedule::CliffLinear { start_ts, .. } => *start_ts,
        }
    }

    pub fn end_ts(&self) -> i64 {
        match &self.schedule {
            VestingSchedule::Immediate => 0,
            VestingSchedule::Linear { end_ts, .. } => *end_ts,
            VestingSchedule::Cliff { cliff_ts } => *cliff_ts,
            VestingSchedule::CliffLinear { end_ts, .. } => *end_ts,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = AddDirectRecipientBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .recipient(self.recipient.pubkey())
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(self.recipient_bump)
            .amount(self.amount)
            .schedule(self.schedule.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "AddDirectRecipient",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        ctx: &TestContext,
        wrong_authority: &Keypair,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = AddDirectRecipientBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(wrong_authority.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .recipient(self.recipient.pubkey())
            .mint(self.mint)
            .distribution_vault(self.distribution_vault)
            .authority_token_account(self.authority_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .bump(self.recipient_bump)
            .amount(self.amount)
            .schedule(self.schedule.clone());

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone()],
            name: "AddDirectRecipient",
        }
    }
}

pub struct AddDirectRecipientSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    schedule: Option<VestingSchedule>,
}

impl<'a> AddDirectRecipientSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self { ctx, token_program: TOKEN_PROGRAM_ID, amount: DEFAULT_RECIPIENT_AMOUNT, schedule: None }
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

    pub fn build(self) -> AddDirectRecipientSetup {
        let mut distribution_builder = CreateDirectDistributionSetup::builder(self.ctx);
        if self.token_program == TOKEN_2022_PROGRAM_ID {
            distribution_builder = distribution_builder.token_2022();
        }
        let distribution_setup = distribution_builder.build();

        let instruction = distribution_setup.build_instruction(self.ctx);
        instruction.send_expect_success(self.ctx);

        let recipient = self.ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_direct_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let current_ts = self.ctx.get_current_timestamp();
        let schedule =
            self.schedule.unwrap_or(VestingSchedule::Linear { start_ts: current_ts, end_ts: current_ts + 86400 * 365 });

        let authority_token_account = self.ctx.create_ata_for_program_with_balance(
            &distribution_setup.authority.pubkey(),
            &distribution_setup.mint.pubkey(),
            self.amount,
            &self.token_program,
        );

        AddDirectRecipientSetup {
            authority: distribution_setup.authority,
            distribution_pda: distribution_setup.distribution_pda,
            recipient,
            recipient_pda,
            recipient_bump,
            amount: self.amount,
            schedule,
            token_program: self.token_program,
            mint: distribution_setup.mint.pubkey(),
            distribution_vault: distribution_setup.distribution_vault,
            authority_token_account,
        }
    }
}

pub struct AddDirectRecipientFixture;

impl InstructionTestFixture for AddDirectRecipientFixture {
    const INSTRUCTION_NAME: &'static str = "AddDirectRecipient";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = AddDirectRecipientSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: payer (handled by TestContext)
    /// 1: authority
    fn required_signers() -> &'static [usize] {
        &[0, 1]
    }

    /// Account indices that must be writable:
    /// 0: payer (handled by TestContext)
    /// 2: distribution
    /// 3: recipient_account
    /// 6: distribution_vault
    /// 7: authority_token_account
    fn required_writable() -> &'static [usize] {
        &[0, 2, 3, 6, 7]
    }

    fn system_program_index() -> Option<usize> {
        Some(8)
    }

    fn current_program_index() -> Option<usize> {
        Some(11)
    }

    fn data_len() -> usize {
        // discriminator(1) + bump(1) + amount(8) + Linear schedule(17) = 27
        1 + 1 + 8 + 17
    }
}
