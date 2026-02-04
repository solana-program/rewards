use pinocchio::error::ProgramError;

use crate::traits::InstructionData;

/// Instruction data for EmitEvent
///
/// This instruction has no data - event data is stored in the instruction data itself.
pub struct EmitEventData {}

impl<'a> TryFrom<&'a [u8]> for EmitEventData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(_data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl<'a> InstructionData<'a> for EmitEventData {
    const LEN: usize = 0;
}
