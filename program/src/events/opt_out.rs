use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct OptOutEvent {
    pub reward_pool: Address,
    pub user: Address,
    pub rewards_claimed: u64,
}

impl EventDiscriminator for OptOutEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::OptedOut as u8;
}

impl EventSerialize for OptOutEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.rewards_claimed.to_le_bytes());
        data
    }
}

impl OptOutEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8; // reward_pool + user + rewards_claimed

    #[inline(always)]
    pub fn new(reward_pool: Address, user: Address, rewards_claimed: u64) -> Self {
        Self { reward_pool, user, rewards_claimed }
    }
}
