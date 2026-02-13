//! # Rewards Program
//!
//! A Solana program for managing token rewards distributions with vesting
//! schedules and claimable allocations.
//!
//! ## Features
//! - Direct distributions with explicit recipient lists
//! - Merkle distributions with proof-based claims
//! - Continuous reward pools with proportional distribution
//! - Configurable vesting schedules for direct and merkle distributions
//! - Claimable rewards and revocation flows across all distribution types
//! - Token-2022 extension blocking
//!
//! ## Architecture
//! Built with Pinocchio (no_std). Clients auto-generated via Codama.

#![no_std]

extern crate alloc;

use pinocchio::address::declare_id;

pub mod errors;
pub mod traits;
pub mod utils;

pub mod events;
pub mod instructions;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

declare_id!("3bwhqHWmDDh3gYZ8gFpSFBwwtyiSTfhRnWZo34nSjohX");

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Rewards Program",
    project_url: "https://github.com/solana-program/rewards",
    contacts: "link:https://github.com/solana-program/rewards/security/advisories/new",
    policy: "https://github.com/solana-program/rewards/security/policy",
    source_code: "https://github.com/solana-program/rewards"
}
