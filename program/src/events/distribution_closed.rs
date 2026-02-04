use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct DistributionClosedEvent {
    pub distribution: Address,
}

impl EventDiscriminator for DistributionClosedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::DistributionClosed as u8;
}

impl EventSerialize for DistributionClosedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.distribution.as_ref());
        data
    }
}

impl DistributionClosedEvent {
    pub const DATA_LEN: usize = 32; // distribution

    #[inline(always)]
    pub fn new(distribution: Address) -> Self {
        Self { distribution }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_distribution_closed_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);

        let event = DistributionClosedEvent::new(distribution);

        assert_eq!(event.distribution, distribution);
    }

    #[test]
    fn test_distribution_closed_event_to_bytes_inner() {
        let distribution = Address::new_from_array([1u8; 32]);
        let event = DistributionClosedEvent::new(distribution);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), DistributionClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], distribution.as_ref());
    }

    #[test]
    fn test_distribution_closed_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let event = DistributionClosedEvent::new(distribution);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + DistributionClosedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::DistributionClosed as u8);
        assert_eq!(&bytes[9..41], distribution.as_ref());
    }
}
