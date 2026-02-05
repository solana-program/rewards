use alloc::vec::Vec;
use pinocchio::error::ProgramError;

use crate::{
    require_len,
    traits::{InstructionData, VestingParams},
    utils::VestingScheduleType,
};

/// Instruction data for ClaimMerkle
///
/// Variable-length due to merkle proof array
pub struct ClaimMerkleData {
    /// Bump for the claim PDA
    pub claim_bump: u8,
    /// Total amount allocated to this claimant (from merkle leaf)
    pub total_amount: u64,
    /// Vesting schedule type (from merkle leaf)
    pub schedule_type: u8,
    /// Vesting start timestamp (from merkle leaf)
    pub start_ts: i64,
    /// Vesting end timestamp (from merkle leaf)
    pub end_ts: i64,
    /// Amount to claim (0 = claim all available)
    pub amount: u64,
    /// Merkle proof (variable length)
    pub proof: Vec<[u8; 32]>,
}

impl<'a> TryFrom<&'a [u8]> for ClaimMerkleData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // Minimum length: claim_bump (1) + total_amount (8) + schedule_type (1) + start_ts (8) + end_ts (8) + amount (8) + proof_len (4) = 38
        require_len!(data, Self::LEN);

        let claim_bump = data[0];
        let total_amount = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let schedule_type = data[9];
        let start_ts = i64::from_le_bytes(data[10..18].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let end_ts = i64::from_le_bytes(data[18..26].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let amount = u64::from_le_bytes(data[26..34].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let proof_len =
            u32::from_le_bytes(data[34..Self::LEN].try_into().map_err(|_| ProgramError::InvalidInstructionData)?)
                as usize;

        let expected_len = Self::LEN + proof_len * 32;
        require_len!(data, expected_len);

        let mut proof = Vec::with_capacity(proof_len);
        for i in 0..proof_len {
            let start = Self::LEN + i * 32;
            let end = start + 32;
            let hash: [u8; 32] = data[start..end].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
            proof.push(hash);
        }

        Ok(Self { claim_bump, total_amount, schedule_type, start_ts, end_ts, amount, proof })
    }
}

impl<'a> InstructionData<'a> for ClaimMerkleData {
    // claim_bump (1) + total_amount (8) + schedule_type (1) + start_ts (8) + end_ts (8) + amount (8) + proof_len (4) = 38
    const LEN: usize = 38;
}

impl VestingParams for ClaimMerkleData {
    #[inline(always)]
    fn total_amount(&self) -> u64 {
        self.total_amount
    }

    #[inline(always)]
    fn start_ts(&self) -> i64 {
        self.start_ts
    }

    #[inline(always)]
    fn end_ts(&self) -> i64 {
        self.end_ts
    }

    #[inline(always)]
    fn schedule_type(&self) -> VestingScheduleType {
        VestingScheduleType::from_u8(self.schedule_type).unwrap_or(VestingScheduleType::Immediate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_data_no_proof() -> Vec<u8> {
        let mut data = Vec::with_capacity(38);
        data.push(255); // claim_bump
        data.extend_from_slice(&1000u64.to_le_bytes()); // total_amount
        data.push(1); // schedule_type = Linear
        data.extend_from_slice(&100i64.to_le_bytes()); // start_ts
        data.extend_from_slice(&200i64.to_le_bytes()); // end_ts
        data.extend_from_slice(&500u64.to_le_bytes()); // amount
        data.extend_from_slice(&0u32.to_le_bytes()); // proof_len = 0
        data
    }

    fn create_valid_data_with_proof() -> Vec<u8> {
        let mut data = Vec::with_capacity(38 + 64);
        data.push(255); // claim_bump
        data.extend_from_slice(&1000u64.to_le_bytes()); // total_amount
        data.push(0); // schedule_type = Immediate
        data.extend_from_slice(&100i64.to_le_bytes()); // start_ts
        data.extend_from_slice(&200i64.to_le_bytes()); // end_ts
        data.extend_from_slice(&0u64.to_le_bytes()); // amount = 0 (claim all)
        data.extend_from_slice(&2u32.to_le_bytes()); // proof_len = 2
        data.extend_from_slice(&[1u8; 32]); // proof[0]
        data.extend_from_slice(&[2u8; 32]); // proof[1]
        data
    }

    #[test]
    fn test_try_from_valid_data_no_proof() {
        let data = create_valid_data_no_proof();
        let result = ClaimMerkleData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.claim_bump, 255);
        assert_eq!(parsed.total_amount, 1000);
        assert_eq!(parsed.schedule_type, 1); // Linear
        assert_eq!(parsed.start_ts, 100);
        assert_eq!(parsed.end_ts, 200);
        assert_eq!(parsed.amount, 500);
        assert!(parsed.proof.is_empty());
    }

    #[test]
    fn test_try_from_valid_data_with_proof() {
        let data = create_valid_data_with_proof();
        let result = ClaimMerkleData::try_from(&data[..]);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.claim_bump, 255);
        assert_eq!(parsed.total_amount, 1000);
        assert_eq!(parsed.schedule_type, 0); // Immediate
        assert_eq!(parsed.start_ts, 100);
        assert_eq!(parsed.end_ts, 200);
        assert_eq!(parsed.amount, 0);
        assert_eq!(parsed.proof.len(), 2);
        assert_eq!(parsed.proof[0], [1u8; 32]);
        assert_eq!(parsed.proof[1], [2u8; 32]);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 30];
        let result = ClaimMerkleData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_try_from_proof_too_short() {
        let mut data = create_valid_data_no_proof();
        // Claim to have 2 proof elements but don't include them
        data[34..38].copy_from_slice(&2u32.to_le_bytes());
        let result = ClaimMerkleData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = create_valid_data_no_proof();
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_vesting_params_linear() {
        let data = create_valid_data_no_proof();
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.schedule_type(), VestingScheduleType::Linear);
    }

    #[test]
    fn test_vesting_params_immediate() {
        let data = create_valid_data_with_proof();
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.schedule_type(), VestingScheduleType::Immediate);
    }
}
