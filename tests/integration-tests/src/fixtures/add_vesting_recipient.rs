use rewards_program_client::instructions::AddVestingRecipientBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::{CreateVestingDistributionSetup, LINEAR_SCHEDULE};
use crate::utils::{
    find_event_authority_pda, find_vesting_recipient_pda, InstructionTestFixture, TestContext, TestInstruction,
    PROGRAM_ID,
};

pub const DEFAULT_RECIPIENT_AMOUNT: u64 = 1_000_000;

pub struct AddVestingRecipientSetup {
    pub authority: Keypair,
    pub distribution_pda: Pubkey,
    pub recipient: Keypair,
    pub recipient_pda: Pubkey,
    pub recipient_bump: u8,
    pub amount: u64,
    pub schedule_type: u8,
    pub start_ts: i64,
    pub end_ts: i64,
    pub token_program: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub distribution_amount: u64,
}

impl AddVestingRecipientSetup {
    pub fn builder(ctx: &mut TestContext) -> AddVestingRecipientSetupBuilder<'_> {
        AddVestingRecipientSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn from_distribution_setup(ctx: &mut TestContext, distribution_setup: &CreateVestingDistributionSetup) -> Self {
        let instruction = distribution_setup.build_instruction(ctx);
        instruction.send_expect_success(ctx);

        let recipient = ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_vesting_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let current_ts = ctx.get_current_timestamp();

        Self {
            authority: distribution_setup.authority.insecure_clone(),
            distribution_pda: distribution_setup.distribution_pda,
            recipient,
            recipient_pda,
            recipient_bump,
            amount: DEFAULT_RECIPIENT_AMOUNT,
            schedule_type: LINEAR_SCHEDULE,
            start_ts: current_ts,
            end_ts: current_ts + 86400 * 365,
            token_program: distribution_setup.token_program,
            mint: distribution_setup.mint.pubkey(),
            vault: distribution_setup.vault,
            distribution_amount: distribution_setup.amount,
        }
    }

    pub fn build_instruction(&self, ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = AddVestingRecipientBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(self.authority.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .recipient(self.recipient.pubkey())
            .mint(self.mint)
            .vault(self.vault)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .program(PROGRAM_ID)
            .bump(self.recipient_bump)
            .amount(self.amount)
            .schedule_type(self.schedule_type)
            .start_ts(self.start_ts)
            .end_ts(self.end_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.authority.insecure_clone()],
            name: "AddVestingRecipient",
        }
    }

    pub fn build_instruction_with_wrong_authority(
        &self,
        ctx: &TestContext,
        wrong_authority: &Keypair,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = AddVestingRecipientBuilder::new();
        builder
            .payer(ctx.payer.pubkey())
            .authority(wrong_authority.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .recipient(self.recipient.pubkey())
            .mint(self.mint)
            .vault(self.vault)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .program(PROGRAM_ID)
            .bump(self.recipient_bump)
            .amount(self.amount)
            .schedule_type(self.schedule_type)
            .start_ts(self.start_ts)
            .end_ts(self.end_ts);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_authority.insecure_clone()],
            name: "AddVestingRecipient",
        }
    }
}

pub struct AddVestingRecipientSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    schedule_type: u8,
    start_ts: Option<i64>,
    end_ts: Option<i64>,
    distribution_amount: Option<u64>,
}

impl<'a> AddVestingRecipientSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self {
            ctx,
            token_program: TOKEN_PROGRAM_ID,
            amount: DEFAULT_RECIPIENT_AMOUNT,
            schedule_type: LINEAR_SCHEDULE,
            start_ts: None,
            end_ts: None,
            distribution_amount: None,
        }
    }

    pub fn token_2022(mut self) -> Self {
        self.token_program = TOKEN_2022_PROGRAM_ID;
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = amount;
        self
    }

    pub fn schedule_type(mut self, schedule_type: u8) -> Self {
        self.schedule_type = schedule_type;
        self
    }

    pub fn start_ts(mut self, ts: i64) -> Self {
        self.start_ts = Some(ts);
        self
    }

    pub fn end_ts(mut self, ts: i64) -> Self {
        self.end_ts = Some(ts);
        self
    }

    pub fn distribution_amount(mut self, amount: u64) -> Self {
        self.distribution_amount = Some(amount);
        self
    }

    pub fn build(self) -> AddVestingRecipientSetup {
        let mut distribution_builder = CreateVestingDistributionSetup::builder(self.ctx);
        if self.token_program == TOKEN_2022_PROGRAM_ID {
            distribution_builder = distribution_builder.token_2022();
        }
        if let Some(dist_amount) = self.distribution_amount {
            distribution_builder = distribution_builder.amount(dist_amount);
        }
        let distribution_setup = distribution_builder.build();
        let distribution_amount = distribution_setup.amount;

        let instruction = distribution_setup.build_instruction(self.ctx);
        instruction.send_expect_success(self.ctx);

        let recipient = self.ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_vesting_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let current_ts = self.ctx.get_current_timestamp();
        let start_ts = self.start_ts.unwrap_or(current_ts);
        let end_ts = self.end_ts.unwrap_or(current_ts + 86400 * 365);

        AddVestingRecipientSetup {
            authority: distribution_setup.authority,
            distribution_pda: distribution_setup.distribution_pda,
            recipient,
            recipient_pda,
            recipient_bump,
            amount: self.amount,
            schedule_type: self.schedule_type,
            start_ts,
            end_ts,
            token_program: self.token_program,
            mint: distribution_setup.mint.pubkey(),
            vault: distribution_setup.vault,
            distribution_amount,
        }
    }
}

pub struct AddVestingRecipientFixture;

impl InstructionTestFixture for AddVestingRecipientFixture {
    const INSTRUCTION_NAME: &'static str = "AddVestingRecipient";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = AddVestingRecipientSetup::new(ctx);
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
    fn required_writable() -> &'static [usize] {
        &[0, 2, 3]
    }

    fn system_program_index() -> Option<usize> {
        Some(7)
    }

    fn current_program_index() -> Option<usize> {
        Some(10)
    }

    fn data_len() -> usize {
        1 + 1 + 8 + 1 + 8 + 8 // discriminator + bump + amount + schedule_type + start_ts + end_ts
    }
}
