# Template Adapter — Quick Start

Copy this directory to create a new yield adapter for the Solana Yield Adapter Standard (SYAS) using the **Quasar** framework.

## Step 1: Copy & rename

```bash
cp -r programs/adapter-template programs/adapter-myprotocol
cd programs/adapter-myprotocol
# Rename all "template"/"Template" references in filenames and code
# e.g., sed -i 's/template/myprotocol/g; s/Template/Myprotocol/g' src/**/*
```

## Step 2: Generate a program ID

```bash
solana-keygen grind --starts-with MYPR
# Update declare_id! in src/lib.rs with the generated keypair
```

## Step 3: Set your protocol ID

In `src/lib.rs`, set `EXTERNAL_PROGRAM_ID` to your target protocol's program ID.

## Step 4: Customize seeds

In `src/state.rs`, update the PDA seeds:

```rust
#[seeds(b"myprotocol_vault_state")]       // rename
#[seeds(b"myprotocol_vault_authority")]   // rename
```

## Step 5: Implement protocol CPI

In `src/protocol.rs`, implement `on_deposit` and `on_withdraw` with real `invoke_signed`
calls. Use the Kamino or MarginFi adapters as reference examples.

## Step 6: Register in workspace

Edit `Cargo.toml` at the repo root, adding `"programs/adapter-myprotocol"` to the `members` list.

## Step 7: Add to build

Append `adapter-myprotocol` to the `PROGRAMS` array in:
- `scripts/build-quasar.sh`
- `scripts/build-sbf.sh`
- `scripts/build-idls.sh`

## Step 8: Add to Anchor.toml

Add the program ID to both `[programs.localnet]` and `[programs.devnet]`:

```toml
adapter_myprotocol = "YOUR_PROGRAM_ID"
```

## Step 9: Add test

Copy `tests/adapters/template.test.ts` to `tests/adapters/myprotocol.test.ts`
and update the references. Then add it to `.mocharc.yml`.

## Step 10: Build & test

```bash
# Rust-level QuasarSVM test
cd programs/adapter-myprotocol && quasar build && cargo test

# Full integration test (requires localnet validator)
npm run test
```

## Reference adapters

For complete examples with real CPI, see:
- `programs/adapter-kamino/` — Kamino K-Lend (share-based lending vault)
- `programs/adapter-marginfi/` — MarginFi v2 (JIT risk engine)
- `programs/adapter-jupiter/` — Jupiter Perps JLP pool
- `programs/adapter-maple/` — Maple syrupUSDC (no CPI, yield-bearing SPL token)
- `programs/adapter-drift/` — Drift v2 Insurance Fund
