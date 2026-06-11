<h1 align="center">
  <code>syas-quasar</code>
</h1>
<p align="center">
  Solana Yield Adapter Standard — <strong>Quasar</strong> port for framework speed benchmark.
</p>

> Full port from Anchor to <a href="https://github.com/blueshift-gg/quasar">Quasar</a> — 86% smaller binaries, in-process QuasarSVM tests, and framework speed benchmarks vs Anchor. See <a href="./README-QUASAR.md">README-QUASAR.md</a> and <a href="./MIGRATION-STATUS.md">MIGRATION-STATUS.md</a>. Run <code>npm run benchmark</code> to reproduce results.

<p align="center">
  Anchor baseline: <code>../solana-yield-adapter-standard</code> · <a href="https://quasar-lang.com/docs">Quasar docs</a>
</p>

<div align="center">

![Solana](https://img.shields.io/badge/Solana-2.2.20-9945FF?style=for-the-badge&logo=solana)
![Quasar](https://img.shields.io/badge/Quasar-beta-9945FF?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-2021-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-Apache_2.0-green?style=for-the-badge)

[README-QUASAR](./README-QUASAR.md) · [Migration status](./MIGRATION-STATUS.md) · [Benchmark](./scripts/benchmark-frameworks.sh)

</div>

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Reference Adapters](#reference-adapters)
5. [Project Structure](#project-structure)
6. [Testing](#testing)
7. [Deployment](#deployment)
8. [Framework Benchmark: Anchor vs Quasar](#framework-benchmark-anchor-vs-quasar)
9. [Adapter Standard Specification](#adapter-standard-specification)
10. [Build Your Own Adapter](#build-your-own-adapter)
11. [Security Model](#security-model)
12. [Contributing](#contributing)
13. [License](#license)

---

## Overview

The **Solana Yield Adapter Standard** defines a minimal, composable interface for interacting with yield-bearing protocols on Solana. Think of it as an **ERC-4626 for Solana** — a universal adapter layer that lets wallets, aggregators, and dApps interact with any yield source through three simple instructions:

| Instruction | Description |
|---|---|
| **`deposit(amount)`** | Deposit underlying tokens into the yield source |
| **`withdraw(amount)`** | Withdraw underlying tokens from the yield source |
| **`current_value()`** | Query the current value of a position |

### Why?

Every DeFi protocol on Solana has its own unique interface. This means:
- Aggregators must write custom integration code for each protocol
- Wallets can't display yield positions in a standardized way
- New protocols face adoption friction due to integration overhead

The Yield Adapter Standard solves this by providing a **single interface** that all yield protocols can implement.

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                    Solana Yield Adapter Standard                        │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                     Core Dispatcher Program                          ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐                      ││
│  │  │ deposit() │  │withdraw()│  │current_value()│                      ││
│  │  └─────┬─────┘  └─────┬────┘  └──────┬───────┘                      ││
│  │        └──────────────┴───────────────┘                              ││
│  │                    │  Validates & Routes                              ││
│  └────────────────────┼────────────────────────────────────────────────┘│
│                       │                                                  │
│  ┌────────────────────▼────────────────────────────────────────────────┐│
│  │                    Adapter Registry (Governance-Gated)               ││
│  │  propose_adapter() → approve_adapter() → revoke_adapter()           ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                       │                                                  │
│  ┌────────────────────▼────────────────────────────────────────────────┐│
│  │                     Reference Adapters                               ││
│  │  ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌───────┐ ┌──────────────┐ ││
│  │  │ Kamino  │ │ MarginFi │ │ Jupiter  │ │ Maple │ │    Drift     │ ││
│  │  │  USDC   │ │   USDC   │ │   LP     │ │ Syrup │ │ Insurance    │ ││
│  │  └─────────┘ └──────────┘ └──────────┘ └───────┘ └──────────────┘ ││
│  └─────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────┘
```

### Components

| Component | Description |
|---|---|
| **Yield Adapter Trait** | Shared crate defining the standard interface, types, events, and errors |
| **Yield Dispatcher** | Router that validates adapters and tracks user positions |
| **Adapter Registry** | Governance-gated on-chain registry for adapter approval |
| **Reference Adapters** | Five production-grade adapter implementations |

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (2.2.20+) or [Agave 3.1.10](https://github.com/anza-xyz/agave) (for Quasar builds)
- [Quasar](https://github.com/blueshift-gg/quasar) CLI (framework)
- [Node.js](https://nodejs.org/) (18+)

### Build

```bash
git clone https://github.com/max-de-bug/syas-quasar.git
cd syas-quasar

npm install

# Build all 8 programs (.so in target/deploy/)
# Requires Agave 3.1.x: agave-install init 3.1.10
npm run build
```

### Test

```bash
# QuasarSVM unit tests (fast, no validator)
cargo test --package adapter-registry
cargo test --package yield-dispatcher
# ... or per-program: cd programs/<name> && quasar test

# Mainnet-fork integration tests (solana-test-validator)
npm run test:fork

# Mainnet-fork tests via Surfpool (alternative, JIT account fetching)
bash scripts/run-fork-surfpool.sh
```

### Deploy to Devnet

```bash
./scripts/deploy-devnet.sh
```

See [SUBMISSION.md](SUBMISSION.md) and [docs/REFERENCE_IMPLEMENTATION.md](docs/REFERENCE_IMPLEMENTATION.md) for program IDs and devnet deployment references.

---

## Reference Adapters

| Adapter | Protocol | Underlying | Model | Status |
|---|---|---|---|---|
| **Kamino USDC** | [Kamino Finance](https://kamino.finance) | USDC | Share-based reference vault | ✅ Complete |
| **MarginFi USDC** | [MarginFi](https://marginfi.com) | USDC | Share-based reference vault | ✅ Complete |
| **Jupiter LP** | [Jupiter](https://jup.ag) | USDC | Share-based reference vault | ✅ Complete |
| **Maple Syrup** | [Maple Finance](https://maple.finance) | USDC | syrupUSDC-style reference | ✅ Complete |
| **Drift Insurance** | [Drift Protocol](https://drift.trade) | USDC | IF staking (13d cooldown) | ✅ Complete |
| **Template** | — | — | Scaffold for new adapters | ✅ Scaffold |

> **Note**: Maple and Drift adapters demonstrate correct interface compliance. Maple operates primarily on EVM chains, and Drift's protocol status may affect live CPI availability. The template adapter is a scaffold — copy it to build a new adapter in under a day.

---

## Project Structure

```
syas-quasar/
├── crates/
│   └── yield-adapter-trait/     # Core interface definitions (shared crate)
├── programs/
│   ├── yield-dispatcher/        # Router with standardized interface
│   ├── adapter-registry/        # Governance-gated adapter registry
│   ├── adapter-kamino/          # Kamino USDC adapter
│   ├── adapter-marginfi/        # MarginFi USDC adapter
│   ├── adapter-jupiter/         # Jupiter LP adapter
│   ├── adapter-maple/           # Maple Syrup adapter
│   ├── adapter-drift/           # Drift Insurance Fund adapter
│   └── adapter-template/        # Scaffold for new adapters
├── tests/
│   ├── helpers/                 # Shared test utilities
│   ├── fixtures/                # Fork test account snapshots
│   ├── adapters/                # Per-adapter integration tests
│   ├── registry.test.ts         # Registry governance tests
│   └── dispatcher.test.ts       # Dispatcher routing tests
├── scripts/
│   ├── build-quasar.sh          # Quasar build orchestrator
│   ├── run-mainnet-fork-tests.sh# solana-test-validator fork runner
│   ├── run-fork-surfpool.sh     # Surfpool fork runner (alternative)
│   ├── deploy-devnet.sh         # Devnet deployment
│   ├── setup-fork-usdc-fixture.sh
│   └── gen-fork-usdc-fixture.mjs
├── docs/
│   ├── ADAPTER_STANDARD.md      # Formal specification
│   ├── BUILD_YOUR_OWN_ADAPTER.md# Developer guide
│   └── REFERENCE_IMPLEMENTATION.md
├── Surfpool.toml                # Surfpool configuration
├── txtx.yml                     # Surfpool runbook manifest
├── runbooks/                    # Surfpool deployment runbooks
├── docs-site/                   # Mintlify documentation site
├── Anchor.toml
├── Cargo.toml
└── README.md
```

---

## Testing

Three test layers, each covering the same `deposit → current_value → withdraw` lifecycle:

### 1. Rust Unit Tests (QuasarSVM, no validator)

```bash
# Per-program QuasarSVM tests — in-process, milliseconds
cd programs/adapter-registry && quasar test     # 3 tests
cd programs/yield-dispatcher && quasar test     # 6 tests
cd programs/adapter-kamino   && quasar test     # 2 tests
# ... same for marginfi, jupiter, maple, drift, template
```

Fastest feedback loop — runs without a Solana validator.

### 2. Mainnet-Fork Tests (solana-test-validator)

```bash
npm run test:fork
```

Clones live mainnet programs (Kamino K-Lend, MarginFi v2, Jupiter Perps, Drift v2) and fixture token accounts into `solana-test-validator`, deploys local programs, and runs **22 integration tests** against real on-chain state.

### 3. Mainnet-Fork Tests (Surfpool — alternative)

```bash
bash scripts/run-fork-surfpool.sh
```

Uses [Surfpool](https://surfpool.run) instead of `solana-test-validator`. Surfpool JIT-fetches accounts from mainnet on demand and loads pre-configured snapshots. Requires a `MAINNET_RPC_URL` for custom RPC endpoints.

### Test coverage

| Suite | Tests | What it verifies |
|-------|-------|------------------|
| Registry | 6 | Init, propose, approve, revoke, governance, access control |
| Dispatcher | 5 | Init, deposit/withdraw via CPI, reject unapproved, reject zero |
| Kamino | 2 | Program load + deposit → current_value → withdraw |
| MarginFi | 2 | Program load + deposit → current_value → withdraw |
| Jupiter | 2 | Program load + deposit → current_value → withdraw |
| Maple | 2 | deposit → current_value → withdraw (syrupUSDC) + zero reject |
| Drift | 2 | Program load + deposit → current_value → withdraw |
| Template | 1 | deposit → current_value → withdraw |
| **Total** | **22** | All pass on mainnet fork |

---

## Deployment

### Devnet

```bash
./scripts/deploy-devnet.sh
```

The script will:
1. Generate program keypairs (if needed)
2. Build all programs
3. Deploy registry and dispatcher to devnet
4. Verify deployment
5. Output the program IDs to update in `Anchor.toml`

### Mainnet

For mainnet deployment, use the same flow with `--provider.cluster mainnet-beta` and ensure proper key management and multisig governance.

---

## Framework Benchmark: Anchor vs Quasar

This project is a full port of the Anchor-based [solana-yield-adapter-standard](../solana-yield-adapter-standard) to the [Quasar](https://github.com/blueshift-gg/quasar) framework. Below are head-to-head benchmark results measured on the same hardware.

### Build Time

| Phase | Anchor | Quasar | Δ |
|-------|--------|--------|---|
| Build all 8 programs | 25.6s | 20.2s | **21% faster** |

Quasar's build advantage comes from a leaner compilation pipeline and `#![no_std]` target.

### Binary Size

| Program | Anchor | Quasar | Reduction |
|---------|--------|--------|-----------|
| yield-dispatcher | 345.0 KB | 53.9 KB | **84%** |
| adapter-registry | 202.9 KB | 21.3 KB | **90%** |
| adapter-kamino | 253.7 KB | 35.5 KB | **86%** |
| adapter-marginfi | 252.5 KB | 35.5 KB | **86%** |
| adapter-jupiter | 251.3 KB | 35.5 KB | **86%** |
| adapter-maple | 235.5 KB | 33.7 KB | **86%** |
| adapter-drift | 253.4 KB | 36.8 KB | **85%** |
| adapter-template | 240.8 KB | 37.4 KB | **84%** |
| **Total** | **2.0 MB** | **0.3 MB** | **86% smaller** |

Quasar's `#![no_std]` + zero-copy serialization eliminates the Borsh runtime overhead, yielding dramatically smaller binaries.

### Test Execution

| Test Suite | Anchor | Quasar |
|------------|--------|--------|
| Rust unit tests | — (mocha only) | 21 tests (QuasarSVM) |
| QuasarSVM integration | — | 21 tests (per-program) |
| Localnet (validator) | 13 passing, 27s | QuasarSVM only |
| Mainnet fork | 20 tests | **22 tests** |

QuasarSVM tests run **in-process** without a Solana validator, making them orders of magnitude faster than `anchor test`. Framework-level integration tests (`deposit → current_value → withdraw` lifecycle) complete in milliseconds vs 3+ seconds per adapter via RPC.

### Key Takeaways

| Metric | Anchor | Quasar | Winner |
|--------|--------|--------|--------|
| Build speed | 25.6s | 20.2s | Quasar |
| Binary size (total) | 2.0 MB | 0.3 MB | **Quasar (86% smaller)** |
| Rust test framework | mocha/TS | QuasarSVM | Quasar |
| Validator needed for tests | yes | no | Quasar |
| Test execution speed | ~3s/adapter via RPC | ~ms/adapter in-process | Quasar |
| On-chain CU | baseline | typically lower | Quasar |
| Maturity | stable (Anchor 1.0.1) | beta | Anchor |

**Run it yourself:**
```bash
cd syas-quasar
npm run benchmark
```

See [README-QUASAR.md](./README-QUASAR.md) for detailed migration notes and the QuasarSVM test reference.

---

## Adapter Standard Specification

See [docs/ADAPTER_STANDARD.md](docs/ADAPTER_STANDARD.md) for the full specification.

### TL;DR — Three Instructions

Every compliant adapter MUST implement:

```rust
// 1. Deposit underlying tokens, receive receipt tokens
fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>;

// 2. Burn receipt tokens, receive underlying tokens
fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>;

// 3. Query current value of position
fn current_value(ctx: Context<CurrentValue>) -> Result<()>;
```

Every adapter MUST emit standardized events: `DepositEvent`, `WithdrawEvent`, `CurrentValueEvent`.

---

## Build Your Own Adapter

See [docs/BUILD_YOUR_OWN_ADAPTER.md](docs/BUILD_YOUR_OWN_ADAPTER.md) for a step-by-step guide.

**Target: Ship a working adapter in less than a day.**

```bash
# 1. Scaffold
anchor init my-adapter && cd my-adapter

# 2. Add the trait dependency
# In Cargo.toml: yield-adapter-trait = { git = "..." }

# 3. Implement three instructions: deposit, withdraw, current_value

# 4. Register with the on-chain registry
# Call propose_adapter() → wait for governance approval
```

---

## Security Model

| Layer | Protection |
|---|---|
| **Adapter Registry** | Governance-gated approval prevents malicious adapters |
| **Dispatcher Validation** | All CPI routes are validated against the registry |
| **PDA Authority** | Vault funds are controlled by program-derived addresses |
| **Checked Arithmetic** | All math uses `checked_*` operations to prevent overflows |
| **Event Auditability** | All operations emit standardized events for monitoring |
| **Emergency Pause** | Dispatcher can be paused by governance in emergencies |

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-adapter`)
3. Implement your changes following the adapter standard
4. Add tests for all new functionality
5. Run `cargo fmt && cargo clippy --workspace`
6. Submit a pull request

---

## License

This project is licensed under the Apache License 2.0 — see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built for the Solana ecosystem 🌊**

[Documentation](https://syas.mintlify.app) · [Adapter Standard](docs/ADAPTER_STANDARD.md) · [Report Issue](https://github.com/max-de-bug/solana-yield-adapter-standard/issues)

</div>
