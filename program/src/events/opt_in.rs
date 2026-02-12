use alloc::vec::Vec;
use codama::CodamaType;
use pinocchio::Address;

use crate::traits::{EventDiscriminator, EventDiscriminators, EventSerialize};

#[derive(CodamaType)]
pub struct OptInEvent {
    pub reward_pool: Address,
    pub user: Address,
    pub balance: u64,
}

impl EventDiscriminator for OptInEvent {
    const DISCRIMINATOR: u8 = EventDiscriminators::OptedIn as u8;
}

impl EventSerialize for OptInEvent {
    #[inline(always)]
    fn to_bytes_inner(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::DATA_LEN);
        data.extend_from_slice(self.reward_pool.as_ref());
        data.extend_from_slice(self.user.as_ref());
        data.extend_from_slice(&self.balance.to_le_bytes());
        data
    }
}

impl OptInEvent {
    pub const DATA_LEN: usize = 32 + 32 + 8; // reward_pool + user + balance

    #[inline(always)]
    pub fn new(reward_pool: Address, user: Address, balance: u64) -> Self {
        Self { reward_pool, user, balance }
    }
}
