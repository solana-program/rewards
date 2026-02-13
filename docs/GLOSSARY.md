# Rewards Program Glossary

This glossary maps equivalent concepts across reward classes so contributors can
translate terms quickly without reading unrelated modules.

## Core Term Mapping

| Shared Concept | Direct | Merkle | Continuous |
| --- | --- | --- | --- |
| Parent config account | `DirectDistribution` | `MerkleDistribution` | `RewardPool` |
| Rewards vault ATA | `distribution_vault` | `distribution_vault` | `reward_vault` |
| User-level tracking account | `DirectRecipient` | `MerkleClaim` | `UserRewardAccount` |
| End user key | `recipient` | `claimant` | `user` |
| Claim instruction | `ClaimDirect` | `ClaimMerkle` | `ClaimContinuous` |
| Revoke instruction | `RevokeDirectRecipient` | `RevokeMerkleClaim` | `RevokeUser` |

## Notes

- Direct and Merkle are vesting-based distribution models.
- Continuous is accumulator-based and does not use vesting schedules.
- All classes can use a `Revocation` marker PDA to permanently block future claims or opt-ins for a user in a specific parent account.
