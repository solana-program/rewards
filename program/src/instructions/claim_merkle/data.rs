use alloc::vec::Vec;
use pinocchio::error::ProgramError;

use crate::{
    require_len,
    traits::{InstructionData, VestingParams},
    utils::VestingSchedule,
};

/// Instruction data for ClaimMerkle.
///
/// Fixed-size fields first, then variable-length schedule and proof.
pub struct ClaimMerkleData {
    /// Bump for the claim PDA
    pub claim_bump: u8,
    /// Total amount allocated to this claimant (from merkle leaf)
    pub total_amount: u64,
    /// Amount to claim (0 = claim all available)
    pub amount: u64,
    /// Vesting schedule (from merkle leaf, variable length)
    pub schedule: VestingSchedule,
    /// Merkle proof (variable length)
    pub proof: Vec<[u8; 32]>,
}

impl<'a> TryFrom<&'a [u8]> for ClaimMerkleData {
    type Error = ProgramError;

    #[inline(always)]
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // Minimum length: claim_bump(1) + total_amount(8) + amount(8) + schedule(1) + proof_len(4) = 22
        require_len!(data, Self::LEN);

        // Fixed-size fields at known offsets
        let claim_bump = data[0];
        let total_amount = u64::from_le_bytes(data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
        let amount = u64::from_le_bytes(data[9..17].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        // Variable-length fields
        let (schedule, schedule_len) = VestingSchedule::from_bytes(&data[17..])?;

        let proof_offset = 17 + schedule_len;
        require_len!(data, proof_offset + 4); // proof_len(4)

        let proof_len = u32::from_le_bytes(
            data[proof_offset..proof_offset + 4].try_into().map_err(|_| ProgramError::InvalidInstructionData)?,
        ) as usize;

        let proof_start = proof_offset + 4;
        let expected_len = proof_start + proof_len * 32;
        require_len!(data, expected_len);

        let mut proof = Vec::with_capacity(proof_len);
        for i in 0..proof_len {
            let start = proof_start + i * 32;
            let end = start + 32;
            let hash: [u8; 32] = data[start..end].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
            proof.push(hash);
        }

        Ok(Self { claim_bump, total_amount, amount, schedule, proof })
    }
}

impl<'a> InstructionData<'a> for ClaimMerkleData {
    // claim_bump(1) + total_amount(8) + amount(8) + min_schedule(1) + proof_len(4) = 22
    const LEN: usize = 22;
}

impl VestingParams for ClaimMerkleData {
    #[inline(always)]
    fn total_amount(&self) -> u64 {
        self.total_amount
    }

    #[inline(always)]
    fn vesting_schedule(&self) -> VestingSchedule {
        self.schedule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_data(schedule: VestingSchedule, amount: u64, proof: &[[u8; 32]]) -> Vec<u8> {
        let schedule_bytes = schedule.to_bytes();
        let mut data = Vec::new();
        // Fixed-size fields
        data.push(255); // claim_bump
        data.extend_from_slice(&1000u64.to_le_bytes()); // total_amount
        data.extend_from_slice(&amount.to_le_bytes()); // amount
                                                       // Variable-length fields
        data.extend_from_slice(&schedule_bytes); // schedule
        data.extend_from_slice(&(proof.len() as u32).to_le_bytes()); // proof_len
        for p in proof {
            data.extend_from_slice(p);
        }
        data
    }

    #[test]
    fn test_try_from_immediate_no_proof() {
        let data = build_data(VestingSchedule::Immediate {}, 500, &[]);
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.claim_bump, 255);
        assert_eq!(parsed.total_amount, 1000);
        assert_eq!(parsed.schedule, VestingSchedule::Immediate {});
        assert_eq!(parsed.amount, 500);
        assert!(parsed.proof.is_empty());
    }

    #[test]
    fn test_try_from_linear_with_proof() {
        let schedule = VestingSchedule::Linear { start_ts: 100, end_ts: 200 };
        let proof = [[1u8; 32], [2u8; 32]];
        let data = build_data(schedule, 0, &proof);
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.schedule, schedule);
        assert_eq!(parsed.amount, 0);
        assert_eq!(parsed.proof.len(), 2);
        assert_eq!(parsed.proof[0], [1u8; 32]);
        assert_eq!(parsed.proof[1], [2u8; 32]);
    }

    #[test]
    fn test_try_from_cliff_linear() {
        let schedule = VestingSchedule::CliffLinear { start_ts: 0, cliff_ts: 100, end_ts: 400 };
        let data = build_data(schedule, 250, &[]);
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.schedule, schedule);
        assert_eq!(parsed.amount, 250);
    }

    #[test]
    fn test_try_from_data_too_short() {
        let data = [0u8; 10];
        let result = ClaimMerkleData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_try_from_proof_too_short() {
        let schedule = VestingSchedule::Immediate {};
        let schedule_bytes = schedule.to_bytes();
        let mut data = Vec::new();
        data.push(255); // claim_bump
        data.extend_from_slice(&1000u64.to_le_bytes()); // total_amount
        data.extend_from_slice(&0u64.to_le_bytes()); // amount
        data.extend_from_slice(&schedule_bytes);
        data.extend_from_slice(&2u32.to_le_bytes()); // claim 2 proofs but don't include them
        let result = ClaimMerkleData::try_from(&data[..]);
        assert_eq!(result.err(), Some(ProgramError::InvalidInstructionData));
    }

    #[test]
    fn test_validate_success() {
        let data = build_data(VestingSchedule::Linear { start_ts: 100, end_ts: 200 }, 0, &[]);
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn test_vesting_params_linear() {
        let data = build_data(VestingSchedule::Linear { start_ts: 100, end_ts: 200 }, 500, &[]);
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.vesting_schedule(), VestingSchedule::Linear { start_ts: 100, end_ts: 200 });
    }

    #[test]
    fn test_vesting_params_immediate() {
        let data = build_data(VestingSchedule::Immediate {}, 0, &[]);
        let parsed = ClaimMerkleData::try_from(&data[..]).unwrap();
        assert_eq!(parsed.vesting_schedule(), VestingSchedule::Immediate {});
    }
}
