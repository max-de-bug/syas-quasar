# Solana Yield Adapter Standard (Submission)

## Repository

Publish this directory as a public GitHub repository before submitting the bounty form.

## Toolchain

| Component | Version |
|-----------|---------|
| Framework | **Quasar** |
| Solana CLI / runtime (tests) | **2.2.20** |
| SBF build platform-tools | **Agave 3.1.10** (`agave-install init 3.1.10`) |

Build produces `target/deploy/*.so` via `./scripts/build-quasar.sh` (uses `quasar build` per program).

## Reference / mock positioning

**This is a reference implementation**, not production integrations:

- Adapters implement the **standard trait** (`deposit`, `withdraw`, `current_value`) using **local share-based vaults** and SPL token transfers.
- **No on-chain CPI** into Kamino, MarginFi, Jupiter Perps, Maple, or Drift live programs.
- Mainnet program IDs in code/docs are for **fork visibility tests** and metadata only.
- Maple adapter uses **real syrupUSDC** (yield-bearing SPL token) on mainnet fork; Drift adapter is **illustrative**.

See [docs/REFERENCE_IMPLEMENTATION.md](docs/REFERENCE_IMPLEMENTATION.md) for details.

## Program IDs (localnet / devnet keypairs)

| Program | Address |
|---------|---------|
| `adapter_registry` | `CeyDkRgegNUz2TeFfFjRdL89G9EGGDymiqHoJkeFGcZ4` |
| `yield_dispatcher` | `7oUKys5XKMzD2NmFCZyLDyTF2Hm1VH3qX8jVfZEY4f3r` |
| `adapter_kamino` | `BzuVWb3UgCW6axee6ZNb812D268XrWkJsE7mxkX9b3Kp` |
| `adapter_marginfi` | `FrCvyyGSukMZcLhpU7EneuhfPmqS5p8E2ysnFdwHhopR` |
| `adapter_jupiter` | `2acqkTDi2VQ4FCZVDB8PeMVLVWnREogE5HA2GxvHdWxu` |
| `adapter_maple` | `Ft2Yvaiqwsjvo1yyYEWvt12YCsDB4kjGBd7vrF8RwwjU` |
| `adapter_drift` | `CVfb8T9tf9WEeus4mKWsxTehVezeY9TGwYsSc3JmxWYz` |

Deploy all programs: `./scripts/deploy-devnet.sh` (uses `target/deploy/*` keypairs).

## Devnet (deployed)

| Program | Devnet address | Explorer |
|---------|----------------|----------|
| `adapter_registry` | `CeyDkRgegNUz2TeFfFjRdL89G9EGGDymiqHoJkeFGcZ4` | [view](https://explorer.solana.com/address/CeyDkRgegNUz2TeFfFjRdL89G9EGGDymiqHoJkeFGcZ4?cluster=devnet) |
| `yield_dispatcher` | `7oUKys5XKMzD2NmFCZyLDyTF2Hm1VH3qX8jVfZEY4f3r` | [view](https://explorer.solana.com/address/7oUKys5XKMzD2NmFCZyLDyTF2Hm1VH3qX8jVfZEY4f3r?cluster=devnet) |
| `adapter_kamino` | `BzuVWb3UgCW6axee6ZNb812D268XrWkJsE7mxkX9b3Kp` | [view](https://explorer.solana.com/address/BzuVWb3UgCW6axee6ZNb812D268XrWkJsE7mxkX9b3Kp?cluster=devnet) |
| `adapter_marginfi` | `FrCvyyGSukMZcLhpU7EneuhfPmqS5p8E2ysnFdwHhopR` | [view](https://explorer.solana.com/address/FrCvyyGSukMZcLhpU7EneuhfPmqS5p8E2ysnFdwHhopR?cluster=devnet) |

Program IDs are fixed in `target/deploy/` keypairs. Run `./scripts/deploy-devnet.sh` after funding the deploy wallet.

After deploy, initialize registry and dispatcher from your wallet (see `tests/registry.test.ts` / `tests/dispatcher.test.ts` account layout).

## Test commands

```bash
# Install JS deps
npm install

# Build all programs (.so + IDL)
npm run build

# QuasarSVM unit tests (fast, no validator)
cargo test

# TS integration tests (requires local validator or mainnet fork)
npm test

# Mainnet fork tests (requires cloned programs + USDC fixture)
npm run test:fork
```

Fork setup (first time):

```bash
./scripts/setup-fork-usdc-fixture.sh
./scripts/run-mainnet-fork-tests.sh
```

## Architecture highlights

- **Registry:** propose → approve governance for adapter metadata and mint binding.
- **Dispatcher:** validates `AdapterEntry` is `Approved`, then **CPI** to the matching reference adapter.
- **Adapters:** share-priced vault PDAs; implement `YieldAdapter` trait surface.
- **Tests:** 26 Rust QuasarSVM tests (unit + integration) + 16 TypeScript integration tests (20 on mainnet fork).

## Links

- Spec: [docs/ADAPTER_STANDARD.md](docs/ADAPTER_STANDARD.md)
- Build your own adapter: [docs/BUILD_YOUR_OWN_ADAPTER.md](docs/BUILD_YOUR_OWN_ADAPTER.md)
- Full README: [README.md](README.md)
