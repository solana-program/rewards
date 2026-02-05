use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct RecipientAddedEvent {
    pub distribution: Address,
    pub recipient: Address,
    pub amount: u64,
    pub schedule_type: u8,
    pub start_ts: i64,
    pub end_ts: i64,
}

impl EventDiscriminator for RecipientAddedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::RecipientAdded as u8;
}

impl EventSerialize for RecipientAddedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.distribution.as_ref());
        data.extend_from_slice(self.recipient.as_ref());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.push(self.schedule_type);
        data.extend_from_slice(&self.start_ts.to_le_bytes());
        data.extend_from_slice(&self.end_ts.to_le_bytes());
        data
    }
}

impl RecipientAddedEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8 + 1 + 8 + 8; // distribution + recipient + amount + schedule_type + start_ts + end_ts

    #[inline(always)]
    pub fn new(
        distribution: Address,
        recipient: Address,
        amount: u64,
        schedule_type: u8,
        start_ts: i64,
        end_ts: i64,
    ) -> Self {
        Self { distribution, recipient, amount, schedule_type, start_ts, end_ts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EVENT_IX_TAG_LE;
    use crate::traits::EVENT_DISCRIMINATOR_LEN;

    #[test]
    fn test_recipient_added_event_new() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);

        let event = RecipientAddedEvent::new(distribution, recipient, 1000, 0, 100, 200);

        assert_eq!(event.distribution, distribution);
        assert_eq!(event.recipient, recipient);
        assert_eq!(event.amount, 1000);
        assert_eq!(event.schedule_type, 0);
        assert_eq!(event.start_ts, 100);
        assert_eq!(event.end_ts, 200);
    }

    #[test]
    fn test_recipient_added_event_to_bytes_inner() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let event = RecipientAddedEvent::new(distribution, recipient, 5000, 0, 100, 200);

        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), RecipientAddedEvent::DATA_LEN);
        assert_eq!(&bytes[..32], distribution.as_ref());
        assert_eq!(&bytes[32..64], recipient.as_ref());
        assert_eq!(&bytes[64..72], &5000u64.to_le_bytes());
    }

    #[test]
    fn test_recipient_added_event_to_bytes() {
        let distribution = Address::new_from_array([1u8; 32]);
        let recipient = Address::new_from_array([2u8; 32]);
        let event = RecipientAddedEvent::new(distribution, recipient, 1000, 0, 100, 200);

        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + RecipientAddedEvent::DATA_LEN);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], EventDiscriminators::RecipientAdded as u8);
    }
}
