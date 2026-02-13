//! Merkle distribution instructions.
//!
//! Merkle distributions store only a root on-chain and use proof-based claims
//! to support large recipient sets with constant on-chain state.

pub mod claim;
pub mod close_claim;
pub mod close_distribution;
pub mod create_distribution;
pub mod revoke_claim;
