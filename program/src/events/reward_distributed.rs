use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct RewardDistributedEvent {
    pub reward_pool: Address,
    pub amount: u64,
    pub new_reward_per_token: u128,
}

impl EventDiscriminator for RewardDistributedEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::RewardDistributed as u8;
}

impl EventSerialize for RewardDistributedEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.new_reward_per_token.to_le_bytes());
        data
    }
}

impl RewardDistributedEvent {
    pub const DATA_LEN: usize = 32 + 8 + 16; // reward_pool + amount + new_reward_per_token

    #[inline(always)]
    pub fn new(reward_pool: Address, amount: u64, new_reward_per_token: u128) -> Self {
        Self { reward_pool, amount, new_reward_per_token }
    }
}
