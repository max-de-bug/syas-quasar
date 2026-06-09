# SYAS on Quasar — Framework Speed Benchmark

This directory is a **Quasar port** of [solana-yield-adapter-standard](../solana-yield-adapter-standard) (Anchor). The goal is an apples-to-apples comparison of build, deploy, and test speed on the same mainnet-fork suite.

| Framework | Repo | Toolchain |
|-----------|------|-----------|
| **Anchor** | `../solana-yield-adapter-standard` | Anchor 1.0.1 · Agave 3.1.10 SBF |
| **Quasar** | `syas-quasar` (this repo) | [Quasar](https://github.com/blueshift-gg/quasar) · platform-tools v1.52 |

Docs: [quasar-lang.com](https://quasar-lang.com/docs)

## What we compare

Same four phases as the Anchor baseline:

| Phase | Command (Anchor) | Command (Quasar) |
|-------|------------------|------------------|
| Build | `npm run build` | `npm run build` → `scripts/build-quasar.sh` |
| Fork validator | clone mainnet programs | identical `run-mainnet-fork-tests.sh` |
| Deploy | deploy 7 `.so` to fork | same keypairs, same deploy loop |
| Tests | 20 fork integration tests | same 20 tests (TS clients from Quasar IDL) |

Run both side-by-side:

```bash
./scripts/benchmark-frameworks.sh
```

Results are written to `benchmark-results/` with timestamps.

## Migration status

See [MIGRATION-STATUS.md](./MIGRATION-STATUS.md) for per-program port progress.

**Important:** Quasar uses explicit `#[instruction(discriminator = N)]` (not Anchor SHA256 discriminators). Tests must use **Quasar-generated TypeScript clients** (`quasar client`), not `@anchor-lang/core` workspace types. Logic and account layout stay identical.

## Prerequisites

```bash
# Quasar CLI (from blueshift-gg/quasar)
git clone https://github.com/blueshift-gg/quasar.git ~/projects/quasar
cd ~/projects/quasar/cli && cargo install --path .

# Same Solana toolchain as Anchor SYAS
agave-install init 2.2.20   # fork tests
# Quasar build uses platform-tools v1.52 via cargo build-sbf
```

Set `QUASAR_ROOT` if the quasar repo is not at `../quasar`:

```bash
export QUASAR_ROOT=/path/to/quasar
```

## Quick commands

```bash
npm install
npm run build          # Quasar SBF build (when port complete)
npm test               # Local QuasarSVM / fork tests
npm run test:fork      # Mainnet-fork suite (20 tests)
npm run benchmark      # Compare Anchor vs Quasar timings
```

## Expected differences

| Area | Anchor | Quasar |
|------|--------|--------|
| Build | `cargo build-sbf` + `anchor idl build` | `quasar build` (IDL + clients inline) |
| On-chain | Borsh deserialize, heap | Zero-copy pointer cast, `#![no_std]` |
| Tests | `anchor test` + mocha | `quasar test` or QuasarSVM Rust / TS |
| CU (runtime) | Baseline | Typically lower (see `quasar profile`) |

Wall-clock test time (~3s per adapter flow) is dominated by **RPC confirmations**, not framework choice — expect similar **test execution** times. Differences show up most in **build time** and **on-chain CU**.

## Reference

- Quasar repo: https://github.com/blueshift-gg/quasar
- Migrating from Anchor: https://quasar-lang.com/docs/getting-started/migrating-from-anchor
