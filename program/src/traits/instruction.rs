use pinocchio::{account::AccountView, error::ProgramError};

/// Discriminators for the Rewards Program instructions.
#[repr(u8)]
pub enum RewardsInstructionDiscriminators {
    CreateVestingDistribution = 0,
    AddVestingRecipient = 1,
    ClaimVesting = 2,
    CloseVestingDistribution = 3,
    EmitEvent = 228,
}

impl TryFrom<u8> for RewardsInstructionDiscriminators {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::CreateVestingDistribution),
            1 => Ok(Self::AddVestingRecipient),
            2 => Ok(Self::ClaimVesting),
            3 => Ok(Self::CloseVestingDistribution),
            228 => Ok(Self::EmitEvent),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Marker trait for instruction account structs
///
/// Implementors should use TryFrom<&'a [AccountView]> for parsing
pub trait InstructionAccounts<'a>: Sized + TryFrom<&'a [AccountView], Error = ProgramError> {}

/// Marker trait for instruction data structs
///
/// Implementors should use TryFrom<&'a [u8]> for parsing
pub trait InstructionData<'a>: Sized + TryFrom<&'a [u8], Error = ProgramError> {
    /// Expected length of instruction data
    const LEN: usize;

    /// Validates the instruction data. Override for custom validation.
    fn validate(&self) -> Result<(), ProgramError> {
        Ok(())
    }
}

/// Full instruction combining accounts and data
///
/// Implementors get automatic TryFrom<(&'a [u8], &'a [AccountView])>
pub trait Instruction<'a>: Sized {
    type Accounts: InstructionAccounts<'a>;
    type Data: InstructionData<'a>;

    fn accounts(&self) -> &Self::Accounts;
    fn data(&self) -> &Self::Data;

    /// Parse instruction from data and accounts tuple
    #[inline(always)]
    fn parse(data: &'a [u8], accounts: &'a [AccountView]) -> Result<Self, ProgramError>
    where
        Self: From<(Self::Accounts, Self::Data)>,
    {
        let accounts = Self::Accounts::try_from(accounts)?;
        let data = Self::Data::try_from(data)?;
        Ok(Self::from((accounts, data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discriminator_try_from_create_vesting_distribution() {
        let result = RewardsInstructionDiscriminators::try_from(0u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CreateVestingDistribution));
    }

    #[test]
    fn test_discriminator_try_from_add_vesting_recipient() {
        let result = RewardsInstructionDiscriminators::try_from(1u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::AddVestingRecipient));
    }

    #[test]
    fn test_discriminator_try_from_claim_vesting() {
        let result = RewardsInstructionDiscriminators::try_from(2u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::ClaimVesting));
    }

    #[test]
    fn test_discriminator_try_from_close_vesting_distribution() {
        let result = RewardsInstructionDiscriminators::try_from(3u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::CloseVestingDistribution));
    }

    #[test]
    fn test_discriminator_try_from_emit_event() {
        let result = RewardsInstructionDiscriminators::try_from(228u8);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), RewardsInstructionDiscriminators::EmitEvent));
    }

    #[test]
    fn test_discriminator_try_from_invalid() {
        let result = RewardsInstructionDiscriminators::try_from(5u8);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));

        let result = RewardsInstructionDiscriminators::try_from(255u8);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));

        let result = RewardsInstructionDiscriminators::try_from(100u8);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));
    }
}
