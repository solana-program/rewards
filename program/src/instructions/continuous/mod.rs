//! Continuous reward pool instructions.
//!
//! Continuous rewards track opted-in balances and distribute rewards
//! proportionally over time via a pool-level accumulator.

pub mod claim;
pub mod close_pool;
pub mod create_pool;
pub mod distribute_reward;
pub mod opt_in;
pub mod opt_out;
pub mod revoke_user;
pub mod set_balance;
pub mod sync_balance;
