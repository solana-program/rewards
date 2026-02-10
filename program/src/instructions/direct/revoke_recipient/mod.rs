pub mod accounts;
pub mod data;
pub mod processor;

pub use crate::instructions::impl_instructions::RevokeDirectRecipient;
pub use accounts::*;
pub use data::*;
pub use processor::process_revoke_direct_recipient;
