use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct DistributionCreatedEvent {
    pub authority: Address,
    pub mint: Address,
    pub seeds: Address,
    pub type_data: DistributionCreatedData,
}

#[derive(Clone, Debug, PartialEq, CodamaType)]
pub enum DistributionCreatedData {
    Direct { initial_funding: u64 },
    Merkle { merkle_root: [u8; 32], total_amount: u64, clawback_ts: i64 },
}

impl DistributionCreatedData {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            DistributionCreatedData::Direct { initial_funding } => {
                let mut data = Vec::with_capacity(1 + 8);
                data.push(0); // Direct variant
                data.extend_from_slice(&initial_funding.to_le_bytes());
                data
            }
            DistributionCreatedData::Merkle { merkle_root, total_amount, clawback_ts } => {
                let mut data = Vec::with_capacity(1 + 32 + 8 + 8);
                data.push(1); // Merkle variant
                data.extend_from_slice(merkle_root);
                data.extend_from_slice(&total_amount.to_le_bytes());
                data.extend_from_slice(&clawback_ts.to_le_bytes());
                data
            }
        }
    }
}

impl EventDiscriminator for DistributionCreatedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::DistributionCreated as u8;
}

impl EventSerialize for DistributionCreatedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let type_data_bytes = self.type_data.to_bytes();
        let mut data = Vec::with_capacity(32 + 32 + 32 + type_data_bytes.len());
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.mint.as_ref());
        data.extend_from_slice(self.seeds.as_ref());
        data.extend_from_slice(&type_data_bytes);
        data
    }
}

impl DistributionCreatedEvent {
    pub const DIRECT_DATA_LEN: usize = 32 + 32 + 32 + 1 + 8; // authority + mint + seeds + variant + initial_funding
    pub const MERKLE_DATA_LEN: usize = 32 + 32 + 32 + 1 + 32 + 8 + 8; // authority + mint + seeds + variant + merkle_root + total_amount + clawback_ts

    #[inline(always)]
    pub fn direct(authority: Address, mint: Address, seeds: Address, initial_funding: u64) -> Self {
        Self { authority, mint, seeds, type_data: DistributionCreatedData::Direct { initial_funding } }
    }

    #[inline(always)]
    pub fn merkle(
        authority: Address,
        mint: Address,
        seeds: Address,
        merkle_root: [u8; 32],
        total_amount: u64,
        clawback_ts: i64,
    ) -> Self {
        Self {
            authority,
            mint,
            seeds,
            type_data: DistributionCreatedData::Merkle { merkle_root, total_amount, clawback_ts },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_distribution_created_event_direct() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);

        let event = DistributionCreatedEvent::direct(authority, mint, seeds, 500);

        assert_eq!(event.authority, authority);
        assert_eq!(event.mint, mint);
        assert_eq!(event.seeds, seeds);
        assert!(matches!(event.type_data, DistributionCreatedData::Direct { initial_funding: 500 }));
    }

    #[test]
    fn test_distribution_created_event_merkle() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);
        let merkle_root = [4u8; 32];

        let event = DistributionCreatedEvent::merkle(authority, mint, seeds, merkle_root, 1000, 1700000000);

        assert_eq!(event.authority, authority);
        assert_eq!(event.mint, mint);
        assert_eq!(event.seeds, seeds);
        match event.type_data {
            DistributionCreatedData::Merkle { merkle_root: root, total_amount, clawback_ts } => {
                assert_eq!(root, merkle_root);
                assert_eq!(total_amount, 1000);
                assert_eq!(clawback_ts, 1700000000);
            }
            _ => panic!("Expected Merkle variant"),
        }
    }

    #[test]
    fn test_distribution_created_event_direct_to_bytes_inner() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);
        let event = DistributionCreatedEvent::direct(authority, mint, seeds, 500);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), DistributionCreatedEvent::DIRECT_DATA_LEN);
        assert_eq!(&bytes[..32], authority.as_ref());
        assert_eq!(&bytes[32..64], mint.as_ref());
        assert_eq!(&bytes[64..96], seeds.as_ref());
        assert_eq!(bytes[96], 0); // Direct variant
        assert_eq!(&bytes[97..105], &500u64.to_le_bytes());
    }

    #[test]
    fn test_distribution_created_event_merkle_to_bytes_inner() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);
        let merkle_root = [4u8; 32];
        let event = DistributionCreatedEvent::merkle(authority, mint, seeds, merkle_root, 1000, 1700000000);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), DistributionCreatedEvent::MERKLE_DATA_LEN);
        assert_eq!(&bytes[..32], authority.as_ref());
        assert_eq!(&bytes[32..64], mint.as_ref());
        assert_eq!(&bytes[64..96], seeds.as_ref());
        assert_eq!(bytes[96], 1); // Merkle variant
        assert_eq!(&bytes[97..129], &merkle_root);
        assert_eq!(&bytes[129..137], &1000u64.to_le_bytes());
        assert_eq!(&bytes[137..145], &1700000000i64.to_le_bytes());
    }

    #[test]
    fn test_distribution_created_event_to_bytes() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);
        let event = DistributionCreatedEvent::direct(authority, mint, seeds, 500);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + DistributionCreatedEvent::DIRECT_DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::DistributionCreated as u8);
    }
}
