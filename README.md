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
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (2.2.20+)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) (1.0.1)
- [Node.js](https://nodejs.org/) (18+)

### Build

```bash
# Clone the repository
git clone https://github.com/your-org/solana-yield-adapter-standard.git
cd solana-yield-adapter-standard

# Install toolchain (Solana 2.2.20 + Anchor 1.0.1)
./scripts/install-toolchain.sh

# Install dependencies
yarn install

# Build all programs (.so in target/deploy/)
# Requires Agave 3.1.x platform-tools for SBF: agave-install init 3.1.10
npm run build
```

### Test

```bash
# Run unit tests (local validator, legacy mode)
npm test

# Run mainnet-fork integration tests
npm run test:fork
```

See [SUBMISSION.md](SUBMISSION.md) and [docs/REFERENCE_IMPLEMENTATION.md](docs/REFERENCE_IMPLEMENTATION.md) for bounty submission notes (reference adapters, program IDs).

### Deploy to Devnet

```bash
./scripts/deploy-devnet.sh
```

---

## Reference Adapters

| Adapter | Protocol | Underlying | Model | Status |
|---|---|---|---|---|
| **Kamino USDC** | [Kamino Finance](https://kamino.finance) | USDC | Share-based reference vault | 🔶 Reference |
| **MarginFi USDC** | [MarginFi](https://marginfi.com) | USDC | Share-based reference vault | 🔶 Reference |
| **Jupiter LP** | [Jupiter](https://jup.ag) | USDC | Share-based reference vault | 🔶 Reference |
| **Maple Syrup** | [Maple Finance](https://maple.finance) | USDC | syrupUSDC-style reference | 🔶 Reference |
| **Drift Insurance** | [Drift Protocol](https://drift.trade) | USDC | IF staking (13d cooldown) | 🔶 Reference |

> **Note**: Maple and Drift adapters are reference implementations demonstrating correct interface compliance. Maple operates primarily on EVM chains, and Drift's protocol status may affect live CPI availability.

---

## Project Structure

```
solana-yield-adapter-standard/
├── crates/
│   └── yield-adapter-trait/     # Core interface definitions (shared crate)
├── programs/
│   ├── yield-dispatcher/        # Router with standardized interface
│   ├── adapter-registry/        # Governance-gated adapter registry
│   ├── adapter-kamino/          # Kamino USDC adapter
│   ├── adapter-marginfi/        # MarginFi USDC adapter
│   ├── adapter-jupiter/         # Jupiter LP adapter
│   ├── adapter-maple/           # Maple Syrup adapter
│   └── adapter-drift/           # Drift Insurance Fund adapter
├── tests/
│   ├── helpers/                 # Shared test utilities
│   ├── registry.test.ts         # Registry governance tests
│   └── dispatcher.test.ts       # Dispatcher routing tests
├── scripts/
│   ├── run-mainnet-fork-tests.sh
│   └── deploy-devnet.sh
├── docs/
│   ├── ADAPTER_STANDARD.md      # Formal specification
│   └── BUILD_YOUR_OWN_ADAPTER.md # Developer guide
├── docs-site/                   # Mintlify documentation site
├── Anchor.toml
├── Cargo.toml
└── README.md
```

---

## Testing

### Unit Tests

```bash
anchor test
```

Tests cover:
- **Registry**: Initialize → Propose → Approve → Revoke → Transfer governance
- **Dispatcher**: Initialize → Deposit → Withdraw → Current value → Error cases
- **Adapters**: Deposit → Verify shares → Withdraw → Verify balances

### Mainnet-Fork Tests

```bash
./scripts/run-mainnet-fork-tests.sh
```

Clones live program state from mainnet (Kamino, MarginFi, Drift) and runs integration tests against real on-chain state.

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
| Rust unit tests | — (mocha only) | 19 tests, <0.1s |
| QuasarSVM integration | — | 21 tests (per-program) |
| Localnet (validator) | 13 passing, 27s | QuasarSVM only |
| Mainnet fork | 21 tests | same test suite |

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
