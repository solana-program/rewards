use codama::CodamaType;
use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, CodamaType)]
pub enum RevokeMode {
    NonVested {},
    Full {},
}

impl RevokeMode {
    pub fn from_byte(byte: u8) -> Result<Self, ProgramError> {
        match byte {
            0 => Ok(RevokeMode::NonVested {}),
            1 => Ok(RevokeMode::Full {}),
            _ => Err(RewardsProgramError::InvalidRevokeMode.into()),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            RevokeMode::NonVested {} => 0,
            RevokeMode::Full {} => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revoke_mode_from_byte_non_vested() {
        let mode = RevokeMode::from_byte(0).unwrap();
        assert_eq!(mode, RevokeMode::NonVested {});
    }

    #[test]
    fn test_revoke_mode_from_byte_full() {
        let mode = RevokeMode::from_byte(1).unwrap();
        assert_eq!(mode, RevokeMode::Full {});
    }

    #[test]
    fn test_revoke_mode_from_byte_invalid() {
        let result = RevokeMode::from_byte(2);
        assert_eq!(result.err(), Some(ProgramError::Custom(RewardsProgramError::InvalidRevokeMode as u32)));
    }

    #[test]
    fn test_revoke_mode_to_byte() {
        assert_eq!(RevokeMode::NonVested {}.to_byte(), 0);
        assert_eq!(RevokeMode::Full {}.to_byte(), 1);
    }

    #[test]
    fn test_revoke_mode_roundtrip() {
        for byte in 0..=1 {
            let mode = RevokeMode::from_byte(byte).unwrap();
            assert_eq!(mode.to_byte(), byte);
        }
    }
}
