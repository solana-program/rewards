use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct VestingDistributionCreatedEvent {
    pub authority: Address,
    pub mint: Address,
    pub seeds: Address,
    pub initial_funding: u64,
}

impl EventDiscriminator for VestingDistributionCreatedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::VestingDistributionCreated as u8;
}

impl EventSerialize for VestingDistributionCreatedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.authority.as_ref());
        data.extend_from_slice(self.mint.as_ref());
        data.extend_from_slice(self.seeds.as_ref());
        data.extend_from_slice(&self.initial_funding.to_le_bytes());
        data
    }
}

impl VestingDistributionCreatedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 32 + 8; // authority + mint + seeds + initial_funding

    #[inline(always)]
    pub fn new(authority: Address, mint: Address, seeds: Address, initial_funding: u64) -> Self {
        Self { authority, mint, seeds, initial_funding }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_vesting_distribution_created_event_new() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);

        let event = VestingDistributionCreatedEvent::new(authority, mint, seeds, 500);

        assert_eq!(event.authority, authority);
        assert_eq!(event.mint, mint);
        assert_eq!(event.seeds, seeds);
        assert_eq!(event.initial_funding, 500);
    }

    #[test]
    fn test_vesting_distribution_created_event_to_bytes_inner() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);
        let event = VestingDistributionCreatedEvent::new(authority, mint, seeds, 500);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), VestingDistributionCreatedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], authority.as_ref());
        assert_eq!(&bytes[32..64], mint.as_ref());
        assert_eq!(&bytes[64..96], seeds.as_ref());
    }

    #[test]
    fn test_vesting_distribution_created_event_to_bytes() {
        let authority = Address::new_from_array([1u8; 32]);
        let mint = Address::new_from_array([2u8; 32]);
        let seeds = Address::new_from_array([3u8; 32]);
        let event = VestingDistributionCreatedEvent::new(authority, mint, seeds, 500);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + VestingDistributionCreatedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::VestingDistributionCreated as u8);
    }
}
