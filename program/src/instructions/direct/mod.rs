//! Direct distribution instructions.
//!
//! Direct distributions store recipient allocations on-chain and support
//! authority-managed recipient updates after distribution creation.

pub mod add_recipient;
pub mod claim;
pub mod close_distribution;
pub mod close_recipient;
pub mod create_distribution;
pub mod revoke_recipient;
