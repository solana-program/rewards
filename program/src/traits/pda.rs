use alloc::vec::Vec;
use pinocchio::{account::AccountView, cpi::Seed, error::ProgramError, Address};

/// PDA seed generation tied to state structs
pub trait PdaSeeds {
    /// Static prefix seed (e.g., b"distribution")
    const PREFIX: &'static [u8];

    /// Generate seeds for PDA derivation (without bump)
    /// Used for find_program_address
    fn seeds(&self) -> Vec<&[u8]>;

    /// Generate seeds with bump for signing
    /// Used for invoke_signed
    fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>>;

    /// Derive PDA address from seeds
    #[inline(always)]
    fn derive_address(&self, program_id: &Address) -> (Address, u8) {
        let seeds = self.seeds();
        Address::find_program_address(&seeds, program_id)
    }

    /// Validate that account matches derived PDA
    #[inline(always)]
    fn validate_pda(&self, account: &AccountView, program_id: &Address, expected_bump: u8) -> Result<(), ProgramError> {
        let (derived, bump) = self.derive_address(program_id);
        if bump != expected_bump {
            return Err(ProgramError::InvalidSeeds);
        }
        if account.address() != &derived {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(())
    }

    /// Validate that account address matches derived PDA, returns canonical bump
    #[inline(always)]
    fn validate_pda_address(&self, account: &AccountView, program_id: &Address) -> Result<u8, ProgramError> {
        let (derived, bump) = self.derive_address(program_id);
        if account.address() != &derived {
            return Err(ProgramError::InvalidSeeds);
        }
        Ok(bump)
    }
}

/// Extension trait for account types that store their PDA bump.
/// Provides convenience methods that use the stored bump value.
pub trait PdaAccount: PdaSeeds {
    /// Returns the stored bump seed for this account's PDA
    fn bump(&self) -> u8;

    /// Validate that account matches derived PDA using stored bump
    #[inline(always)]
    fn validate_self(&self, account: &AccountView, program_id: &Address) -> Result<(), ProgramError> {
        self.validate_pda(account, program_id, self.bump())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ID;
    use alloc::vec;

    struct TestPda {
        pub seed: Address,
    }

    impl PdaSeeds for TestPda {
        const PREFIX: &'static [u8] = b"test";

        fn seeds(&self) -> Vec<&[u8]> {
            vec![Self::PREFIX, self.seed.as_ref()]
        }

        fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
            vec![Seed::from(Self::PREFIX), Seed::from(self.seed.as_ref()), Seed::from(&bump[..])]
        }
    }

    struct TestPdaAccount {
        pub seed: Address,
        pub bump: u8,
    }

    impl PdaSeeds for TestPdaAccount {
        const PREFIX: &'static [u8] = b"test_account";

        fn seeds(&self) -> Vec<&[u8]> {
            vec![Self::PREFIX, self.seed.as_ref()]
        }

        fn seeds_with_bump<'a>(&'a self, bump: &'a [u8; 1]) -> Vec<Seed<'a>> {
            vec![Seed::from(Self::PREFIX), Seed::from(self.seed.as_ref()), Seed::from(&bump[..])]
        }
    }

    impl PdaAccount for TestPdaAccount {
        fn bump(&self) -> u8 {
            self.bump
        }
    }

    #[test]
    fn test_derive_address_deterministic() {
        let pda = TestPda { seed: Address::new_from_array([1u8; 32]) };

        let (address1, bump1) = pda.derive_address(&ID);
        let (address2, bump2) = pda.derive_address(&ID);

        assert_eq!(address1, address2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_derive_address_different_seeds() {
        let pda1 = TestPda { seed: Address::new_from_array([1u8; 32]) };
        let pda2 = TestPda { seed: Address::new_from_array([2u8; 32]) };

        let (address1, _) = pda1.derive_address(&ID);
        let (address2, _) = pda2.derive_address(&ID);

        assert_ne!(address1, address2);
    }

    #[test]
    fn test_seeds_returns_correct_structure() {
        let pda = TestPda { seed: Address::new_from_array([5u8; 32]) };
        let seeds = pda.seeds();

        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0], TestPda::PREFIX);
        assert_eq!(seeds[1], pda.seed.as_ref());
    }

    #[test]
    fn test_pda_account_bump() {
        let account = TestPdaAccount { seed: Address::new_from_array([1u8; 32]), bump: 254 };
        assert_eq!(account.bump(), 254);
    }

    #[test]
    fn test_pda_account_inherits_pda_seeds() {
        let account = TestPdaAccount { seed: Address::new_from_array([1u8; 32]), bump: 255 };
        let seeds = account.seeds();
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0], TestPdaAccount::PREFIX);
    }

    #[test]
    fn test_pda_account_derive_address() {
        let account = TestPdaAccount { seed: Address::new_from_array([1u8; 32]), bump: 255 };
        let (address, _bump) = account.derive_address(&ID);
        assert!(!address.as_ref().iter().all(|&b| b == 0));
    }
}
