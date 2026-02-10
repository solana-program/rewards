use alloc::vec;
use alloc::vec::Vec;
use codama::CodamaAccount;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address};

use crate::traits::{
    AccountParse, AccountSerialize, AccountSize, AccountValidation, Discriminator, PdaSeeds,
    RewardsAccountDiscriminators, Versioned,
};
use crate::{require_account_len, validate_discriminator};

/// MerkleRevocation account state
///
/// Minimal PDA recording that a claimant's allocation has been revoked
/// from a merkle distribution. Its existence blocks future claims.
///
/// # PDA Seeds
/// `[b"merkle_revocation", distribution.as_ref(), claimant.as_ref()]`
#[derive(Clone, Debug, PartialEq, CodamaAccount)]
pub struct MerkleRevocation {
    pub bump: u8,
}

impl Discriminator for MerkleRevocation {
    const DISCRIMINATOR: u8 = RewardsAccountDiscriminators::MerkleRevocation as u8;
}

impl Versioned for MerkleRevocation {
    const VERSION: u8 = 1;
}

impl AccountSize for MerkleRevocation {
    const DATA_LEN: usize = 1; // bump
}

impl AccountParse for MerkleRevocation {
    fn parse_from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        require_account_len!(data, Self::LEN);
        validate_discriminator!(data, Self::DISCRIMINATOR);

        // Skip discriminator (byte 0) and version (byte 1)
        let data = &data[2..];

        let bump = data[0];

        Ok(Self { bump })
    }
}

impl AccountSerialize for MerkleRevocation {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.push(self.bump);
        data
    }
}

impl AccountValidation for MerkleRevocation {}

/// Seed helper for deriving MerkleRevocation PDA without having the full state
pub struct MerkleRevocationSeeds {
    pub distribution: Address,
    pub claimant: Address,
}

impl PdaSeeds for MerkleRevocationSeeds {
    const PREFIX: &'static [u8] = b"merkle_revocation";

    #[inline(always)]
    fn seeds(&self) -> Vec<&[u8]> {
        vec![Self::PREFIX, self.distribution.as_ref(), self.claimant.as_ref()]
    }

    #[inline(always)]
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
        vec![
            Seed::from(Self::PREFIX),
            Seed::from(self.distribution.as_ref()),
            Seed::from(self.claimant.as_ref()),
            Seed::from(bump.as_slice()),
        ]
    }
}

impl MerkleRevocation {
    #[inline(always)]
    pub fn new(bump: u8) -> Self {
        Self { bump }
    }

    #[inline(always)]
    pub fn from_account(
        data: &[u8],
        account: &AccountView,
        program_id: &Address,
        distribution: &Address,
        claimant: &Address,
    ) -> Result<Self, ProgramError> {
        let state = Self::parse_from_bytes(data)?;
        let seeds = MerkleRevocationSeeds { distribution: *distribution, claimant: *claimant };
        seeds.validate_pda(account, program_id, state.bump)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_revocation() -> MerkleRevocation {
        MerkleRevocation::new(255)
    }

    #[test]
    fn test_merkle_revocation_new() {
        let revocation = create_test_revocation();
        assert_eq!(revocation.bump, 255);
    }

    #[test]
    fn test_merkle_revocation_to_bytes_inner() {
        let revocation = create_test_revocation();
        let bytes = revocation.to_bytes_inner();

        assert_eq!(bytes.len(), MerkleRevocation::DATA_LEN);
        assert_eq!(bytes[0], 255); // bump
    }

    #[test]
    fn test_merkle_revocation_to_bytes() {
        let revocation = create_test_revocation();
        let bytes = revocation.to_bytes();

        assert_eq!(bytes.len(), MerkleRevocation::LEN);
        assert_eq!(bytes[0], MerkleRevocation::DISCRIMINATOR);
        assert_eq!(bytes[1], MerkleRevocation::VERSION);
        assert_eq!(bytes[2], 255); // bump
    }

    #[test]
    fn test_roundtrip_serialization() {
        let revocation = create_test_revocation();

        let bytes = revocation.to_bytes();
        let deserialized = MerkleRevocation::parse_from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.bump, revocation.bump);
    }

    #[test]
    fn test_merkle_revocation_seeds_pda_seeds() {
        let seeds = MerkleRevocationSeeds {
            distribution: Address::new_from_array([1u8; 32]),
            claimant: Address::new_from_array([2u8; 32]),
        };
        let pda_seeds = seeds.seeds();
        assert_eq!(pda_seeds.len(), 3);
        assert_eq!(pda_seeds[0], MerkleRevocationSeeds::PREFIX);
        assert_eq!(pda_seeds[1], seeds.distribution.as_ref());
        assert_eq!(pda_seeds[2], seeds.claimant.as_ref());
    }

    #[test]
    fn test_account_size() {
        assert_eq!(MerkleRevocation::DATA_LEN, 1);
        assert_eq!(MerkleRevocation::LEN, 3);
    }
}
