# Vesting Concepts

Vesting applies to Direct and Merkle distributions. Continuous pools do not use
vesting schedules.

## Supported Schedules

- `Immediate`: all tokens are claimable immediately.
- `Linear`: tokens unlock linearly between `start_ts` and `end_ts`.
- `Cliff`: no unlock until `cliff_ts`, then full unlock.
- `CliffLinear`: no unlock until `cliff_ts`, then linear unlock between
  `start_ts` and `end_ts`.

## Where Vesting Is Used

- Direct recipient claims: `ClaimDirect`
- Merkle claimant claims: `ClaimMerkle`
- Revocation settlement in vesting-based distributions

## Practical Guidance

- Use `Immediate` for short campaigns or immediate drops.
- Use `Linear` for continuous unlock programs.
- Use `Cliff` for milestone-gated unlock.
- Use `CliffLinear` for delayed start plus progressive unlock.

## Related References

- Distribution docs: `docs/DIRECT.md`, `docs/MERKLE.md`
- Program README vesting table: `README.md`
