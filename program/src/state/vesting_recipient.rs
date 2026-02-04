use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{
    account::AccountView,
    cpi::{Seed, Signer},
    error::ProgramError,
    Address,
};

use crate::errors::RewardsProgramError;
use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::utils::{calculate_linear_unlock, VestingScheduleType};
use crate::{assert_no_padding, require_account_len, validate_discriminator};

/// VestingRecipient account state
///
/// Represents a recipient's vesting allocation within a distribution.
/// Each recipient has their own vesting schedule (type, start, end).
///
/// # PDA Seeds
/// `[b"vesting_recipient", distribution.as_ref(), recipient.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct VestingRecipient {
    pub bump: u8,
    pub schedule_type: u8,
    _padding: [u8; 6],
    pub distribution: Address,
    pub recipient: Address,
    pub total_amount: u64,
    pub claimed_amount: u64,
    pub start_ts: i64,
    pub end_ts: i64,
}

assert_no_padding!(VestingRecipient, 1 + 1 + 6 + 32 + 32 + 8 + 8 + 8 + 8);

impl Discriminator for VestingRecipient {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::VestingRecipient as u8;
}

impl Versioned for VestingRecipient {
    const VERSION: u8 = 1;
}

impl AccountSize for VestingRecipient {
    const DATA_LEN: usize = 1 + 1 + 6 + 32 + 32 + 8 + 8 + 8 + 8; // 104
}

impl AccountParse for VestingRecipient {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];
        let schedule_type = data[1];
        // Skip padding bytes [2..8]
        let distribution =
            Address::new_from_array(data[8..40].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let recipient =
            Address::new_from_array(data[40..72].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_amount =
            u64::from_le_bytes(data[72..80].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let claimed_amount =
            u64::from_le_bytes(data[80..88].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let start_ts =
            i64::from_le_bytes(data[88..96].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let end_ts = i64::from_le_bytes(data[96..104].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self {
            bump,
            schedule_type,
            _padding: [0u8; 6],
            distribution,
            recipient,
            total_amount,
            claimed_amount,
            start_ts,
            end_ts,
        })
    }
}

impl AccountSerialize for VestingRecipient {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.push(self.schedule_type);
        data.extend_from_slice(&[0u8; 6]); // padding
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.recipient.as_ref());
        data.extend_from_slice(&self.total_amount.to_le_bytes());
        data.extend_from_slice(&self.claimed_amount.to_le_bytes());
        data.extend_from_slice(&self.start_ts.to_le_bytes());
        data.extend_from_slice(&self.end_ts.to_le_bytes());
        data
    }
}

impl AccountValidation for VestingRecipient {}

impl PdaSeeds for VestingRecipient {
    const PREFIX: &'static [u8] = b"vesting_recipient";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.distribution.as_ref(), self.recipient.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.distribution.as_ref()),
            Seed::from(self.recipient.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl VestingRecipient {
    #[inline(always)]
    pub fn new(
        bump: u8,
        distribution: Address,
        recipient: Address,
        total_amount: u64,
        schedule_type: VestingScheduleType,
        start_ts: i64,
        end_ts: i64,
    ) -> Self {
        Self {
            bump,
            schedule_type: schedule_type as u8,
            _padding: [0u8; 6],
            distribution,
            recipient,
            total_amount,
            claimed_amount: 0,
            start_ts,
            end_ts,
        }
    }

    #[inline(always)]
    pub fn from_account(data: &[u8], account: &AccountView, program_id: &Address) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        state.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }

    /// Execute a CPI with this recipient PDA as signer
    #[inline(always)]
    pub fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let seeds = [
            Seed::from(Self::PREFIX),
            Seed::from(self.distribution.as_ref()),
            Seed::from(self.recipient.as_ref()),
            Seed::from(bump_seed.as_slice()),
        ];
        let signers = [Signer::from(&seeds)];
        f(&signers)
    }

    pub fn claimable_amount(&self, unlocked: u64) -> u64 {
        unlocked.saturating_sub(self.claimed_amount)
    }

    pub fn remaining_amount(&self) -> u64 {
        self.total_amount.saturating_sub(self.claimed_amount)
    }

    pub fn schedule_type(&self) -> Option<VestingScheduleType> {
        VestingScheduleType::from_u8(self.schedule_type)
    }

    pub fn calculate_unlocked_amount(&self, current_ts: i64) -> Result<u64, ProgramError> {
        match self.schedule_type() {
            Some(VestingScheduleType::Linear) => {
                calculate_linear_unlock(self.total_amount, self.start_ts, self.end_ts, current_ts)
            }
            None => Ok(0),
        }
    }

    #[inline(always)]
    pub fn validate_distribution(&self, distribution: &Address) -> Result<(), ProgramError> {
        if &self.distribution != distribution {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn validate_recipient(&self, recipient: &Address) -> Result<(), ProgramError> {
        if &self.recipient != recipient {
            return Err(RewardsProgramError::UnauthorizedRecipient.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recipient() -> VestingRecipient {
        VestingRecipient::new(
            255,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            1000,
            VestingScheduleType::Linear,
            100,
            200,
        )
    }

    #[test]
    fn test_vesting_recipient_new() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.bump, 255);
        assert_eq!(recipient.distribution, Address::new_from_array([1u8; 32]));
        assert_eq!(recipient.recipient, Address::new_from_array([2u8; 32]));
        assert_eq!(recipient.total_amount, 1000);
        assert_eq!(recipient.claimed_amount, 0);
        assert_eq!(recipient.schedule_type, VestingScheduleType::Linear as u8);
        assert_eq!(recipient.start_ts, 100);
        assert_eq!(recipient.end_ts, 200);
    }

    #[test]
    fn test_vesting_recipient_to_bytes_inner() {
        let recipient = create_test_recipient();
        let bytes = recipient.to_bytes_inner();

        assert_eq!(bytes.len(), VestingRecipient::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
        assert_eq!(bytes[1], VestingScheduleType::Linear as u8); // schedule_type
        assert_eq!(&bytes[2..8], &[0u8; 6]); // padding
        assert_eq!(&bytes[8..40], &[1u8; 32]); // distribution
        assert_eq!(&bytes[40..72], &[2u8; 32]); // recipient
    }

    #[test]
    fn test_vesting_recipient_to_bytes() {
        let recipient = create_test_recipient();
        let bytes = recipient.to_bytes();

        assert_eq!(bytes.len(), VestingRecipient::LEN);
        assert_eq!(bytes[0], VestingRecipient::DISCRIMINATOR);
        assert_eq!(bytes[1], VestingRecipient::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_vesting_recipient_pda_seeds() {
        let recipient = create_test_recipient();
        let seeds = recipient.seeds();

        assert_eq!(seeds.len(), 3);
        assert_eq!(seeds[0], VestingRecipient::PREFIX);
        assert_eq!(seeds[1], recipient.distribution.as_ref());
        assert_eq!(seeds[2], recipient.recipient.as_ref());
    }

    #[test]
    fn test_claimable_amount() {
        let recipient = create_test_recipient();
        // unlocked 500, claimed 0 -> claimable 500
        assert_eq!(recipient.claimable_amount(500), 500);
    }

    #[test]
    fn test_claimable_amount_with_prior_claims() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 200;
        // unlocked 500, claimed 200 -> claimable 300
        assert_eq!(recipient.claimable_amount(500), 300);
    }

    #[test]
    fn test_remaining_amount() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 300;
        // total 1000, claimed 300 -> remaining 700
        assert_eq!(recipient.remaining_amount(), 700);
    }

    #[test]
    fn test_calculate_unlocked_amount() {
        let recipient = create_test_recipient();
        // start=100, end=200, at ts=150 (midpoint), should unlock 50%
        assert_eq!(recipient.calculate_unlocked_amount(150).unwrap(), 500);
    }

    #[test]
    fn test_calculate_unlocked_before_start() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.calculate_unlocked_amount(50).unwrap(), 0);
    }

    #[test]
    fn test_calculate_unlocked_after_end() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.calculate_unlocked_amount(250).unwrap(), 1000);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 500;

        let bytes = recipient.to_bytes();
        let deserialized = VestingRecipient::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, recipient.bump);
        assert_eq!(deserialized.distribution, recipient.distribution);
        assert_eq!(deserialized.recipient, recipient.recipient);
        assert_eq!(deserialized.total_amount, recipient.total_amount);
        assert_eq!(deserialized.claimed_amount, recipient.claimed_amount);
        assert_eq!(deserialized.schedule_type, recipient.schedule_type);
        assert_eq!(deserialized.start_ts, recipient.start_ts);
        assert_eq!(deserialized.end_ts, recipient.end_ts);
    }

    #[test]
    fn test_schedule_type_valid() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.schedule_type(), Some(VestingScheduleType::Linear));
    }

    #[test]
    fn test_schedule_type_invalid() {
        let mut recipient = create_test_recipient();
        recipient.schedule_type = 255;
        assert_eq!(recipient.schedule_type(), None);
    }

    #[test]
    fn test_calculate_unlocked_amount_invalid_schedule() {
        let mut recipient = create_test_recipient();
        recipient.schedule_type = 255;
        assert_eq!(recipient.calculate_unlocked_amount(150).unwrap(), 0);
    }

    #[test]
    fn test_validate_distribution_success() {
        let recipient = create_test_recipient();
        let dist = Address::new_from_array([1u8; 32]);
        assert!(recipient.validate_distribution(&dist).is_ok());
    }

    #[test]
    fn test_validate_distribution_fail() {
        let recipient = create_test_recipient();
        let wrong_dist = Address::new_from_array([99u8; 32]);
        assert!(recipient.validate_distribution(&wrong_dist).is_err());
    }

    #[test]
    fn test_validate_recipient_success() {
        let recipient = create_test_recipient();
        let recip = Address::new_from_array([2u8; 32]);
        assert!(recipient.validate_recipient(&recip).is_ok());
    }

    #[test]
    fn test_validate_recipient_fail() {
        let recipient = create_test_recipient();
        let wrong_recip = Address::new_from_array([99u8; 32]);
        assert!(recipient.validate_recipient(&wrong_recip).is_err());
    }

    #[test]
    fn test_remaining_amount_full() {
        let recipient = create_test_recipient();
        assert_eq!(recipient.remaining_amount(), 1000);
    }

    #[test]
    fn test_claimable_amount_exceeds_unlocked() {
        let mut recipient = create_test_recipient();
        recipient.claimed_amount = 600;
        // unlocked 500, claimed 600 -> claimable 0 (saturating)
        assert_eq!(recipient.claimable_amount(500), 0);
    }
}
