# Quasar migration status

Track porting from Anchor → Quasar for all 7 on-chain programs.

## Programs

| Program | Anchor LOC | Quasar status | Notes |
|---------|------------|---------------|-------|
| `yield-adapter-trait` | 432 | ✅ Done | `quasar-lang` events/errors/math, `PodU64` routing |
| `adapter-kamino` | 457 | ✅ Done | Build ~8s, 2/2 QuasarSVM tests passing |
| `adapter-registry` | 432 | ✅ Done | Build ~9s, 3/3 QuasarSVM tests passing |
| `yield-dispatcher` | 1,088 | ✅ Done | Build ~9s, 6/6 QuasarSVM tests (deposit→value→withdraw × 5 adapters) |
| `adapter-marginfi` | 443 | ✅ Done | Build ~8s, 2/2 QuasarSVM tests passing |
| `adapter-jupiter` | 443 | ✅ Done | Build ~8s, 2/2 QuasarSVM tests passing |
| `adapter-maple` | 458 | ✅ Done | Build ~9s, 2/2 QuasarSVM tests; holds real syrupUSDC (no simulated yield) |
| `adapter-drift` | 478 | ✅ Done | Build ~6s, 2/2 QuasarSVM tests; 13-day unstake cooldown |

## Phase 1 complete

- [x] `crates/yield-adapter-trait` — Quasar `#![no_std]`, `RemainingAccount` API
- [x] `programs/adapter-kamino` — initialize, deposit, withdraw, current_value
- [x] `programs/adapter-registry` — initialize, propose/approve/revoke, transfer_governance
- [x] `programs/yield-dispatcher` — initialize, deposit, withdraw, current_value
- [x] `programs/adapter-marginfi`, `adapter-jupiter`, `adapter-maple`, `adapter-drift`
- [x] Dispatcher CPI routing: all 5 reference adapters
- [x] QuasarSVM: 14 adapter tests + 3 registry + 6 dispatcher = **23 tests**
- [x] Fork test parity — TS tests rewritten with Quasar instruction builders (no Anchor workspace dependency)
- [x] Dispatcher multi-adapter Rust tests — deposit/withdraw/value through marginfi, jupiter, maple, drift CPIs

## Build & test

```bash
cd programs/adapter-kamino && quasar build && quasar test       # 2 tests
cd programs/adapter-marginfi && quasar build && quasar test     # 2 tests
cd programs/adapter-jupiter && quasar build && quasar test       # 2 tests
cd programs/adapter-maple && quasar build && quasar test       # 2 tests
cd programs/adapter-drift && quasar build && quasar test         # 2 tests
cd programs/adapter-registry && quasar build && quasar test     # 3 tests
cd programs/yield-dispatcher && quasar build && quasar test     # 6 tests (all 5 adapters)
```

## Fork test parity (TS integration tests)

- [x] `tests/helpers/quasar-client.ts` — instruction builders with single-byte discriminators + account deserialization
- [x] `tests/helpers/provider.ts` — replaces `AnchorProvider.env()` with plain `@solana/web3.js`
- [x] `tests/helpers/index.ts` — anchor-free token helpers
- [x] `tests/helpers/constants.ts` — all 7 program IDs + mainnet fork constants
- [x] `tests/helpers/adapter.ts` — Quasar instruction builder-based deposit/withdraw flow
- [x] `tests/helpers/dispatcher.ts` — Quasar-based registry + adapter vault setup
- [x] `tests/registry.test.ts` — initialize, propose, approve, revoke, transfer_governance
- [x] `tests/dispatcher.test.ts` — init, deposit/withdraw via Kamino, reject unapproved
- [x] `tests/adapters/kamino.test.ts` — deposit → current_value → withdraw
- [x] `tests/adapters/marginfi.test.ts` — same pattern
- [x] `tests/adapters/jupiter.test.ts` — same pattern
- [x] `tests/adapters/maple.test.ts` — same pattern
- [x] `tests/adapters/drift.test.ts` — same pattern

## Port checklist (all programs migrated)

1. Add `Quasar.toml` in program directory
2. Replace `anchor-lang` / `anchor-spl` → `quasar-lang` / `quasar-spl`
3. `Pubkey` → `Address`, `Context` → `Ctx`, explicit discriminators
4. Account fields → `PodU64` / `PodBool` at runtime (use `.into()` for math)
5. Import `quasar_lang::sysvars::Sysvar` for `Clock::get()`
6. SPL CPI (adapters only): `.transfer(...).invoke()` / `.invoke_signed(&seeds)`
7. String instruction args: `#[max(N)] name: &str` for borrowed wire format
8. Account enums: store as `u8`, use Rust enum helpers in handlers
9. PDA verify on existing accounts: `seeds.verify(account.address(), &crate::ID)?`
10. `idl-build` feature only at IDL time (not in normal SBF deps)
11. Cross-program validation: use `yield-adapter-trait` IDs/layout byte offsets — do not path-dep other program crates (linker conflicts)
12. Deposit path: validate adapter user-position PDA address only (`init(idempotent)` creates account via CPI); withdraw/value require initialized owner

## Key mapping

See [Migrating from Anchor](https://quasar-lang.com/docs/getting-started/migrating-from-anchor).
