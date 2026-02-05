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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_empty_data() {
        let data: [u8; 0] = [];
        let result = EmitEventData::try_from(&data[..]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_from_with_data() {
        let data = [1, 2, 3, 4];
        let result = EmitEventData::try_from(&data[..]);
        assert!(result.is_ok());
    }
}
