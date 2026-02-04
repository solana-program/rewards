use pinocchio::{
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
};

/// Get the current Unix timestamp from the Clock sysvar.
#[inline(always)]
pub fn get_current_timestamp() -> Result<i64, ProgramError> {
    Ok(Clock::get()?.unix_timestamp)
}
