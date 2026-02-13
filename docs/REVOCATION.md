# Revocation Concepts

Revocation is an authority-controlled safety and lifecycle mechanism available
across all reward classes.

## What Revocation Does

- Settles the user according to configured revocation mode.
- Marks the user as revoked via a `Revocation` PDA.
- Prevents future claims or opt-ins for that user under the same parent account.

## Revocation Modes

- `NonVested`: user receives vested/accrued amount, unvested is returned.
- `Full`: all remaining value is returned to authority.

## Instruction Mapping

- Direct: `RevokeDirectRecipient`
- Merkle: `RevokeMerkleClaim`
- Continuous: `RevokeUser`

## Shared State

- Marker account: `program/src/state/revocation.rs`
- Utilities: `program/src/utils/revoke_utils.rs`

## Operational Notes

- Revocation must be enabled by distribution/pool configuration.
- Revocation is parent scoped: revoking a user in one parent account does not
  revoke them globally across unrelated parents.
