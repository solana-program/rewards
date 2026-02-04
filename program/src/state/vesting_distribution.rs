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
use crate::{assert_no_padding, require_account_len, validate_discriminator};

pub const VESTING_DISTRIBUTION_SEED: &[u8] = b"vesting_distribution";

/// VestingDistribution account state
///
/// Represents a vesting distribution configuration that holds tokens
/// to be distributed to recipients according to a vesting schedule.
///
/// # PDA Seeds
/// `[b"vesting_distribution", mint.as_ref(), authority.as_ref(), seeds.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
#[repr(C)]
pub struct VestingDistribution {
    pub bump: u8,
    _padding: [u8; 7],
    pub authority: Address,
    pub mint: Address,
    pub seeds: Address,
    pub total_allocated: u64,
    pub total_claimed: u64,
}

assert_no_padding!(VestingDistribution, 1 + 7 + 32 + 32 + 32 + 8 + 8);

impl Discriminator for VestingDistribution {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::VestingDistribution as u8;
}

impl Versioned for VestingDistribution {
    const VERSION: u8 = 1;
}

impl AccountSize for VestingDistribution {
    const DATA_LEN: usize = 1 + 7 + 32 + 32 + 32 + 8 + 8; // 120
}

impl AccountParse for VestingDistribution {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];
        // Skip padding bytes [1..8]
        let authority =
            Address::new_from_array(data[8..40].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let mint =
            Address::new_from_array(data[40..72].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let seeds =
            Address::new_from_array(data[72..104].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_allocated =
            u64::from_le_bytes(data[104..112].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);
        let total_claimed =
            u64::from_le_bytes(data[112..120].try_into().map_err(|_| RewardsProgramError::InvalidAccountData)?);

        Ok(Self { bump, _padding: [0u8; 7], authority, mint, seeds, total_allocated, total_claimed })
    }
}

impl AccountSerialize for VestingDistribution {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data.extend_from_slice(&[0u8; 7]); // padding
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.mint.as_ref());
        data.extend_from_slice(self.seeds.as_ref());
        data.extend_from_slice(&self.total_allocated.to_le_bytes());
        data.extend_from_slice(&self.total_claimed.to_le_bytes());
        data
    }
}

impl AccountValidation for VestingDistribution {}

impl PdaSeeds for VestingDistribution {
    const PREFIX: &'static [u8] = VESTING_DISTRIBUTION_SEED;

    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.mint.as_ref(), self.authority.as_ref(), self.seeds.as_ref()]
    }

    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.mint.as_ref()),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seeds.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl VestingDistribution {
    #[inline(always)]
    pub fn new(bump: u8, authority: Address, mint: Address, seeds: Address) -> Self {
        Self { bump, _padding: [0u8; 7], authority, mint, seeds, total_allocated: 0, total_claimed: 0 }
    }

    #[inline(always)]
    pub fn from_account(data: &[u8], account: &AccountView, program_id: &Address) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        state.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }

    /// Execute a CPI with this distribution PDA as signer
    #[inline(always)]
    pub fn with_signer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Signer<'_, '_>]) -> R,
    {
        let bump_seed = [self.bump];
        let pda_seeds = [
            Seed::from(VESTING_DISTRIBUTION_SEED),
            Seed::from(self.mint.as_ref()),
            Seed::from(self.authority.as_ref()),
            Seed::from(self.seeds.as_ref()),
            Seed::from(bump_seed.as_slice()),
        ];
        let signers = [Signer::from(&pda_seeds)];
        f(&signers)
    }

    pub fn remaining_unallocated(&self, vault_balance: u64) -> u64 {
        vault_balance.saturating_sub(self.total_allocated.saturating_sub(self.total_claimed))
    }

    #[inline(always)]
    pub fn validate_authority(&self, authority: &Address) -> Result<(), ProgramError> {
        if &self.authority != authority {
            return Err(RewardsProgramError::UnauthorizedAuthority.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_distribution() -> VestingDistribution {
        VestingDistribution::new(
            255,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
        )
    }

    #[test]
    fn test_vesting_distribution_new() {
        let dist = create_test_distribution();
        assert_eq!(dist.bump, 255);
        assert_eq!(dist.total_allocated, 0);
        assert_eq!(dist.total_claimed, 0);
    }

    #[test]
    fn test_vesting_distribution_to_bytes_inner() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes_inner();

        assert_eq!(bytes.len(), VestingDistribution::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
    }

    #[test]
    fn test_vesting_distribution_to_bytes() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();

        assert_eq!(bytes.len(), VestingDistribution::LEN);
        assert_eq!(bytes[0], VestingDistribution::DISCRIMINATOR);
        assert_eq!(bytes[1], VestingDistribution::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_remaining_unallocated() {
        let mut dist = create_test_distribution();
        dist.total_allocated = 500;
        dist.total_claimed = 100;

        // vault has 1000, allocated 500, claimed 100
        // remaining = 1000 - (500 - 100) = 600
        assert_eq!(dist.remaining_unallocated(1000), 600);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let dist = create_test_distribution();
        let bytes = dist.to_bytes();
        let deserialized = VestingDistribution::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, dist.bump);
        assert_eq!(deserialized.authority, dist.authority);
        assert_eq!(deserialized.mint, dist.mint);
        assert_eq!(deserialized.seeds, dist.seeds);
        assert_eq!(deserialized.total_allocated, dist.total_allocated);
        assert_eq!(deserialized.total_claimed, dist.total_claimed);
    }
}
