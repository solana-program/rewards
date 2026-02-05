use const_crypto::sha3::Keccak256;
use pinocchio::{error::ProgramError, Address};

use crate::errors::RewardsProgramError;

/// Leaf prefix to prevent second preimage attacks.
pub const LEAF_PREFIX: &[u8] = &[0];

fn keccak256(data: &[u8]) -> [u8; 32] {
    Keccak256::new().update(data).finalize()
}

/// Compute the merkle leaf hash for a claim.
///
/// The leaf format is:
/// `hash(LEAF_PREFIX || hash(claimant || total_amount || schedule_type || start_ts || end_ts))`
pub fn compute_leaf_hash(
    claimant: &Address,
    total_amount: u64,
    schedule_type: u8,
    start_ts: i64,
    end_ts: i64,
) -> [u8; 32] {
    // Inner hash: hash(claimant || total_amount || schedule_type || start_ts || end_ts)
    let mut inner_data = [0u8; 32 + 8 + 1 + 8 + 8]; // 57 bytes
    inner_data[0..32].copy_from_slice(claimant.as_ref());
    inner_data[32..40].copy_from_slice(&total_amount.to_le_bytes());
    inner_data[40] = schedule_type;
    inner_data[41..49].copy_from_slice(&start_ts.to_le_bytes());
    inner_data[49..57].copy_from_slice(&end_ts.to_le_bytes());

    let inner_hash = keccak256(&inner_data);

    // Outer hash: hash(LEAF_PREFIX || inner_hash)
    let mut outer_data = [0u8; 1 + 32]; // 33 bytes
    outer_data[0..1].copy_from_slice(LEAF_PREFIX);
    outer_data[1..33].copy_from_slice(&inner_hash);

    keccak256(&outer_data)
}

/// Verify a merkle proof against a root.
///
/// The proof is an array of sibling hashes from leaf to root.
/// For each level, if the current hash is less than the sibling,
/// hash(current || sibling), otherwise hash(sibling || current).
/// This sorted pair ordering ensures deterministic tree construction.
pub fn verify_proof(proof: &[[u8; 32]], root: &[u8; 32], leaf: &[u8; 32]) -> bool {
    let mut computed_hash = *leaf;

    for sibling in proof {
        computed_hash = hash_pair(&computed_hash, sibling);
    }

    computed_hash == *root
}

/// Hash two nodes together in sorted order (smaller first).
/// This ensures deterministic tree construction regardless of proof ordering.
fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut data = [0u8; 64];
    if a < b {
        data[0..32].copy_from_slice(a);
        data[32..64].copy_from_slice(b);
    } else {
        data[0..32].copy_from_slice(b);
        data[32..64].copy_from_slice(a);
    }
    keccak256(&data)
}

/// Verify a merkle proof and return error if invalid.
pub fn verify_proof_or_error(proof: &[[u8; 32]], root: &[u8; 32], leaf: &[u8; 32]) -> Result<(), ProgramError> {
    if verify_proof(proof, root, leaf) {
        Ok(())
    } else {
        Err(RewardsProgramError::InvalidMerkleProof.into())
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;

    #[test]
    fn test_compute_leaf_hash_deterministic() {
        let claimant = Address::new_from_array([1u8; 32]);
        let total_amount = 1000u64;
        let schedule_type = 1u8; // Linear
        let start_ts = 100i64;
        let end_ts = 200i64;

        let hash1 = compute_leaf_hash(&claimant, total_amount, schedule_type, start_ts, end_ts);
        let hash2 = compute_leaf_hash(&claimant, total_amount, schedule_type, start_ts, end_ts);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_inputs() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);
        let total_amount = 1000u64;
        let schedule_type = 1u8;
        let start_ts = 100i64;
        let end_ts = 200i64;

        let hash1 = compute_leaf_hash(&claimant1, total_amount, schedule_type, start_ts, end_ts);
        let hash2 = compute_leaf_hash(&claimant2, total_amount, schedule_type, start_ts, end_ts);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_amounts() {
        let claimant = Address::new_from_array([1u8; 32]);
        let schedule_type = 1u8;
        let start_ts = 100i64;
        let end_ts = 200i64;

        let hash1 = compute_leaf_hash(&claimant, 1000, schedule_type, start_ts, end_ts);
        let hash2 = compute_leaf_hash(&claimant, 2000, schedule_type, start_ts, end_ts);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_timestamps() {
        let claimant = Address::new_from_array([1u8; 32]);
        let total_amount = 1000u64;
        let schedule_type = 1u8;

        let hash1 = compute_leaf_hash(&claimant, total_amount, schedule_type, 100, 200);
        let hash2 = compute_leaf_hash(&claimant, total_amount, schedule_type, 100, 300);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_leaf_hash_different_schedule_types() {
        let claimant = Address::new_from_array([1u8; 32]);
        let total_amount = 1000u64;
        let start_ts = 100i64;
        let end_ts = 200i64;

        let hash1 = compute_leaf_hash(&claimant, total_amount, 0, start_ts, end_ts); // Immediate
        let hash2 = compute_leaf_hash(&claimant, total_amount, 1, start_ts, end_ts); // Linear

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_pair_commutative() {
        let a = [1u8; 32];
        let b = [2u8; 32];

        let hash1 = hash_pair(&a, &b);
        let hash2 = hash_pair(&b, &a);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_pair_deterministic() {
        let a = [1u8; 32];
        let b = [2u8; 32];

        let hash1 = hash_pair(&a, &b);
        let hash2 = hash_pair(&a, &b);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_proof_single_leaf() {
        // A tree with a single leaf has the leaf as the root
        let claimant = Address::new_from_array([1u8; 32]);
        let leaf = compute_leaf_hash(&claimant, 1000, 1, 100, 200);
        let root = leaf;
        let proof: Vec<[u8; 32]> = vec![];

        assert!(verify_proof(&proof, &root, &leaf));
    }

    #[test]
    fn test_verify_proof_two_leaves() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);

        let leaf1 = compute_leaf_hash(&claimant1, 1000, 1, 100, 200);
        let leaf2 = compute_leaf_hash(&claimant2, 2000, 1, 100, 200);

        let root = hash_pair(&leaf1, &leaf2);

        // Proof for leaf1 is [leaf2]
        assert!(verify_proof(&[leaf2], &root, &leaf1));

        // Proof for leaf2 is [leaf1]
        assert!(verify_proof(&[leaf1], &root, &leaf2));
    }

    #[test]
    fn test_verify_proof_four_leaves() {
        let leaves: Vec<[u8; 32]> = (0..4)
            .map(|i| {
                let mut addr = [0u8; 32];
                addr[0] = i;
                compute_leaf_hash(&Address::new_from_array(addr), 1000 * (i as u64 + 1), 1, 100, 200)
            })
            .collect();

        // Build tree:
        //        root
        //       /    \
        //     n01    n23
        //    /  \    /  \
        //   l0  l1  l2  l3

        let n01 = hash_pair(&leaves[0], &leaves[1]);
        let n23 = hash_pair(&leaves[2], &leaves[3]);
        let root = hash_pair(&n01, &n23);

        // Proof for leaf0: [leaf1, n23]
        assert!(verify_proof(&[leaves[1], n23], &root, &leaves[0]));

        // Proof for leaf1: [leaf0, n23]
        assert!(verify_proof(&[leaves[0], n23], &root, &leaves[1]));

        // Proof for leaf2: [leaf3, n01]
        assert!(verify_proof(&[leaves[3], n01], &root, &leaves[2]));

        // Proof for leaf3: [leaf2, n01]
        assert!(verify_proof(&[leaves[2], n01], &root, &leaves[3]));
    }

    #[test]
    fn test_verify_proof_invalid_proof() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);

        let leaf1 = compute_leaf_hash(&claimant1, 1000, 1, 100, 200);
        let leaf2 = compute_leaf_hash(&claimant2, 2000, 1, 100, 200);

        let root = hash_pair(&leaf1, &leaf2);

        // Wrong proof
        let wrong_sibling = [99u8; 32];
        assert!(!verify_proof(&[wrong_sibling], &root, &leaf1));
    }

    #[test]
    fn test_verify_proof_invalid_leaf() {
        let claimant1 = Address::new_from_array([1u8; 32]);
        let claimant2 = Address::new_from_array([2u8; 32]);

        let leaf1 = compute_leaf_hash(&claimant1, 1000, 1, 100, 200);
        let leaf2 = compute_leaf_hash(&claimant2, 2000, 1, 100, 200);

        let root = hash_pair(&leaf1, &leaf2);

        // Wrong leaf
        let wrong_leaf = compute_leaf_hash(&claimant1, 9999, 1, 100, 200);
        assert!(!verify_proof(&[leaf2], &root, &wrong_leaf));
    }

    #[test]
    fn test_verify_proof_or_error_success() {
        let claimant = Address::new_from_array([1u8; 32]);
        let leaf = compute_leaf_hash(&claimant, 1000, 1, 100, 200);
        let root = leaf;
        let proof: Vec<[u8; 32]> = vec![];

        assert!(verify_proof_or_error(&proof, &root, &leaf).is_ok());
    }

    #[test]
    fn test_verify_proof_or_error_failure() {
        let claimant = Address::new_from_array([1u8; 32]);
        let leaf = compute_leaf_hash(&claimant, 1000, 1, 100, 200);
        let wrong_root = [99u8; 32];
        let proof: Vec<[u8; 32]> = vec![];

        assert!(verify_proof_or_error(&proof, &wrong_root, &leaf).is_err());
    }
}
