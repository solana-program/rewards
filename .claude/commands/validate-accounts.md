Validate that the Codama account definitions in `definition.rs` match the actual instruction account structs in each `accounts.rs` file.

## Steps

1. Read `program/src/instructions/definition.rs`. Find the top-level enum decorated with `#[derive(..., CodamaInstructions)]`. For each variant in that enum, extract the ordered list of `#[codama(account(...))]` attributes. For each account, record:
    - **name** (the `name = "..."` value)
    - **signer** (whether `signer` is present in the attribute)
    - **writable** (whether `writable` is present in the attribute)

2. Find all `accounts.rs` files under `program/src/instructions/` by globbing for `program/src/instructions/**/accounts.rs`. Read each one. From each file, extract:
    - The **struct name** (e.g. `CreateDirectDistributionAccounts`)
    - The **struct field names** in declaration order (these determine account order)
    - Which fields have `verify_signer(field_name, ...)` calls
    - Which fields have `verify_writable(field_name, ...)` calls

3. Match each `accounts.rs` struct to its definition.rs variant by stripping the `Accounts` suffix from the struct name and matching against variant names (e.g. `CreateDirectDistributionAccounts` → `CreateDirectDistribution`). If a variant has no matching accounts.rs (like `EmitEvent` if it has no separate file), note it but don't treat it as an error.

4. For each matched pair, compare:
    - **Account count**: number of codama account attributes vs number of struct fields
    - **Account names in order**: codama `name` values vs struct field names. Known equivalent names to treat as matching: `seeds` ↔ `seed`, `rewardsProgram` ↔ `program`. Flag any other name differences.
    - **Signer consistency**: codama says `signer` → accounts.rs should have `verify_signer` for that field (and vice versa)
    - **Writable consistency**: codama says `writable` → accounts.rs should have `verify_writable` for that field (and vice versa)

5. Output a summary table per instruction:

    ```
    ## CreateDirectDistribution
    Accounts: 12 (definition) vs 12 (struct) ✓
    Order: ✓
    Signer checks: ✓
    Writable checks: ✓
    ```

    For mismatches, show the specific difference:

    ```
    ## SomeInstruction
    Accounts: 11 (definition) vs 10 (struct) ✗
      - definition has "foo" at position 5, not found in struct
    Signer checks: ✗
      - "authority" marked signer in definition but no verify_signer call
    ```

6. End with a summary: total instructions checked, total mismatches found.
