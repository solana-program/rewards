use rewards_program_client::instructions::ClaimVestingBuilder;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_interface::ID as TOKEN_PROGRAM_ID;

use crate::fixtures::{
    AddVestingRecipientSetup, CreateVestingDistributionSetup, DEFAULT_RECIPIENT_AMOUNT, LINEAR_SCHEDULE,
};
use crate::utils::{
    find_event_authority_pda, find_vesting_recipient_pda, InstructionTestFixture, TestContext, TestInstruction,
    PROGRAM_ID,
};

pub struct ClaimVestingSetup {
    pub recipient: Keypair,
    pub distribution_pda: Pubkey,
    pub recipient_pda: Pubkey,
    pub recipient_bump: u8,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub recipient_token_account: Pubkey,
    pub token_program: Pubkey,
    pub amount: u64,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl ClaimVestingSetup {
    pub fn builder(ctx: &mut TestContext) -> ClaimVestingSetupBuilder<'_> {
        ClaimVestingSetupBuilder::new(ctx)
    }

    pub fn new(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).build()
    }

    pub fn new_token_2022(ctx: &mut TestContext) -> Self {
        Self::builder(ctx).token_2022().build()
    }

    pub fn from_recipient_setup(
        ctx: &mut TestContext,
        recipient_setup: &AddVestingRecipientSetup,
        warp_to_end: bool,
    ) -> Self {
        let instruction = recipient_setup.build_instruction(ctx);
        instruction.send_expect_success(ctx);

        if warp_to_end {
            ctx.warp_to_timestamp(recipient_setup.end_ts);
        }

        let recipient_token_account = if recipient_setup.token_program == TOKEN_2022_PROGRAM_ID {
            ctx.create_token_2022_account(&recipient_setup.recipient.pubkey(), &recipient_setup.mint)
        } else {
            ctx.create_token_account(&recipient_setup.recipient.pubkey(), &recipient_setup.mint)
        };

        Self {
            recipient: recipient_setup.recipient.insecure_clone(),
            distribution_pda: recipient_setup.distribution_pda,
            recipient_pda: recipient_setup.recipient_pda,
            recipient_bump: recipient_setup.recipient_bump,
            mint: recipient_setup.mint,
            vault: recipient_setup.vault,
            recipient_token_account,
            token_program: recipient_setup.token_program,
            amount: recipient_setup.amount,
            start_ts: recipient_setup.start_ts,
            end_ts: recipient_setup.end_ts,
        }
    }

    pub fn build_instruction(&self, _ctx: &TestContext) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClaimVestingBuilder::new();
        builder
            .recipient(self.recipient.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .mint(self.mint)
            .vault(self.vault)
            .recipient_token_account(self.recipient_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .program(PROGRAM_ID);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![self.recipient.insecure_clone()],
            name: "ClaimVesting",
        }
    }

    pub fn build_instruction_with_wrong_recipient(
        &self,
        _ctx: &TestContext,
        wrong_recipient: &Keypair,
        wrong_token_account: Pubkey,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();
        let (wrong_recipient_pda, _) = find_vesting_recipient_pda(&self.distribution_pda, &wrong_recipient.pubkey());

        let mut builder = ClaimVestingBuilder::new();
        builder
            .recipient(wrong_recipient.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(wrong_recipient_pda)
            .mint(self.mint)
            .vault(self.vault)
            .recipient_token_account(wrong_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .program(PROGRAM_ID);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_recipient.insecure_clone()],
            name: "ClaimVesting",
        }
    }

    pub fn build_instruction_with_wrong_signer(
        &self,
        _ctx: &TestContext,
        wrong_signer: &Keypair,
        wrong_signer_token_account: Pubkey,
    ) -> TestInstruction {
        let (event_authority, _) = find_event_authority_pda();

        let mut builder = ClaimVestingBuilder::new();
        builder
            .recipient(wrong_signer.pubkey())
            .distribution(self.distribution_pda)
            .recipient_account(self.recipient_pda)
            .mint(self.mint)
            .vault(self.vault)
            .recipient_token_account(wrong_signer_token_account)
            .token_program(self.token_program)
            .event_authority(event_authority)
            .program(PROGRAM_ID);

        TestInstruction {
            instruction: builder.instruction(),
            signers: vec![wrong_signer.insecure_clone()],
            name: "ClaimVesting",
        }
    }
}

pub struct ClaimVestingSetupBuilder<'a> {
    ctx: &'a mut TestContext,
    token_program: Pubkey,
    amount: u64,
    start_ts: Option<i64>,
    end_ts: Option<i64>,
    warp_to_end: bool,
}

impl<'a> ClaimVestingSetupBuilder<'a> {
    fn new(ctx: &'a mut TestContext) -> Self {
        Self {
            ctx,
            token_program: TOKEN_PROGRAM_ID,
            amount: DEFAULT_RECIPIENT_AMOUNT,
            start_ts: None,
            end_ts: None,
            warp_to_end: true,
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

    pub fn start_ts(mut self, ts: i64) -> Self {
        self.start_ts = Some(ts);
        self
    }

    pub fn end_ts(mut self, ts: i64) -> Self {
        self.end_ts = Some(ts);
        self
    }

    pub fn warp_to_end(mut self, warp: bool) -> Self {
        self.warp_to_end = warp;
        self
    }

    pub fn build(self) -> ClaimVestingSetup {
        let mut distribution_builder = CreateVestingDistributionSetup::builder(self.ctx).amount(self.amount * 2);
        if self.token_program == TOKEN_2022_PROGRAM_ID {
            distribution_builder = distribution_builder.token_2022();
        }
        let distribution_setup = distribution_builder.build();
        let create_ix = distribution_setup.build_instruction(self.ctx);
        create_ix.send_expect_success(self.ctx);

        let current_ts = self.ctx.get_current_timestamp();
        let start_ts = self.start_ts.unwrap_or(current_ts);
        let end_ts = self.end_ts.unwrap_or(current_ts + 86400 * 365);

        let recipient = self.ctx.create_funded_keypair();
        let (recipient_pda, recipient_bump) =
            find_vesting_recipient_pda(&distribution_setup.distribution_pda, &recipient.pubkey());

        let recipient_setup = AddVestingRecipientSetup {
            authority: distribution_setup.authority.insecure_clone(),
            distribution_pda: distribution_setup.distribution_pda,
            recipient: recipient.insecure_clone(),
            recipient_pda,
            recipient_bump,
            amount: self.amount,
            schedule_type: LINEAR_SCHEDULE,
            start_ts,
            end_ts,
            token_program: self.token_program,
            mint: distribution_setup.mint.pubkey(),
            vault: distribution_setup.vault,
            distribution_amount: distribution_setup.amount,
        };
        let add_recipient_ix = recipient_setup.build_instruction(self.ctx);
        add_recipient_ix.send_expect_success(self.ctx);

        if self.warp_to_end {
            self.ctx.warp_to_timestamp(end_ts);
        }

        let recipient_token_account = if self.token_program == TOKEN_2022_PROGRAM_ID {
            self.ctx.create_token_2022_account(&recipient.pubkey(), &distribution_setup.mint.pubkey())
        } else {
            self.ctx.create_token_account(&recipient.pubkey(), &distribution_setup.mint.pubkey())
        };

        ClaimVestingSetup {
            recipient,
            distribution_pda: distribution_setup.distribution_pda,
            recipient_pda,
            recipient_bump,
            mint: distribution_setup.mint.pubkey(),
            vault: distribution_setup.vault,
            recipient_token_account,
            token_program: self.token_program,
            amount: self.amount,
            start_ts,
            end_ts,
        }
    }
}

pub struct ClaimVestingFixture;

impl InstructionTestFixture for ClaimVestingFixture {
    const INSTRUCTION_NAME: &'static str = "ClaimVesting";

    fn build_valid(ctx: &mut TestContext) -> TestInstruction {
        let setup = ClaimVestingSetup::new(ctx);
        setup.build_instruction(ctx)
    }

    /// Account indices that must be signers:
    /// 0: recipient
    fn required_signers() -> &'static [usize] {
        &[0]
    }

    /// Account indices that must be writable:
    /// 1: distribution
    /// 2: recipient_account
    /// 4: vault
    /// 5: recipient_token_account
    fn required_writable() -> &'static [usize] {
        &[1, 2, 4, 5]
    }

    fn system_program_index() -> Option<usize> {
        None
    }

    fn current_program_index() -> Option<usize> {
        Some(8)
    }

    fn data_len() -> usize {
        1 // discriminator only
    }
}
