# New Reward Class Checklist

Use this checklist when adding a new reward class or introducing a new
instruction family in an existing class.

## 1) Define the class model

- [ ] Define parent state account in `program/src/state/`
- [ ] Define user-level state account(s) in `program/src/state/` if needed
- [ ] Add/extend account discriminators in `program/src/traits/account.rs`
- [ ] Add PDA seed strategy and document naming in `README.md`

## 2) Define instruction surface

- [ ] Add instruction variants in `program/src/instructions/definition.rs`
- [ ] Add instruction data/accounts wiring in `program/src/instructions/impl_instructions.rs`
- [ ] Add discriminator variants and `TryFrom<u8>` mapping in `program/src/traits/instruction.rs`

## 3) Implement instruction modules

- [ ] Add class folder under `program/src/instructions/<class>/`
- [ ] For each instruction, add `accounts.rs`, `data.rs`, `processor.rs`, `mod.rs`
- [ ] Add class-level module docs in `program/src/instructions/<class>/mod.rs`

## 4) Wire runtime dispatch

- [ ] Add processor routing in `program/src/instructions/mod.rs` (`dispatch_instruction`)
- [ ] Confirm `program/src/entrypoint.rs` remains a thin adapter

## 5) Shared concerns

- [ ] Integrate event emission via `program/src/events/` as needed
- [ ] Integrate revocation handling if required (`program/src/state/revocation.rs`)
- [ ] Integrate token-2022 safety checks and authority checks consistently

## 6) Tests and docs

- [ ] Add instruction-level integration tests under `tests/integration-tests/src/`
- [ ] Add fixture support under `tests/integration-tests/src/fixtures/`
- [ ] Update docs in `README.md` and `docs/`
- [ ] Run CU tracking (`just integration-test --with-cu`) and update `docs/CU_BENCHMARKS.md`

## 7) Final verification

- [ ] `just fmt`
- [ ] `just test`
- [ ] Validate generated IDL/client output if instruction schema changed
