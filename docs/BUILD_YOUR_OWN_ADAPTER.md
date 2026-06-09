# Build Your Own Adapter — Developer Guide

> **Goal**: Ship a working, conformant yield adapter in less than a day.

This guide walks you through building a yield adapter from scratch. By the end, you'll have a fully functional adapter that integrates with the Solana Yield Adapter Standard.

---

## Prerequisites

- Rust + Cargo (1.75+)
- [Quasar](https://quasar-lang.com) CLI installed
- Solana CLI (Agave 3.1.10+)
- Node.js 18+
- Familiarity with Solana program development

## Step 1: Scaffold Your Project (15 min)

```bash
# Create a new Quasar project
quasar new my-yield-adapter
cd my-yield-adapter

# Add the trait crate dependency
# In programs/my-yield-adapter/Cargo.toml:
```

```toml
[dependencies]
quasar-lang = "0.1"

# The trait provides shared types, events, and error codes
yield-adapter-trait = { git = "https://github.com/your-org/solana-yield-adapter-standard" }
```

```bash
# Copy the existing Adapter.toml/Quasar.toml from a reference adapter
cp ../solana-yield-adapter-standard/programs/adapter-kamino/Quasar.toml programs/my-yield-adapter/
```

## Step 2: Define Your Vault State (15 min)

Create `src/state.rs`:

```rust
use quasar_lang::prelude::*;

#[account]
pub struct MyVaultState {
    pub authority: Address,
    pub underlying_mint: Address,
    pub total_underlying: u64,
    pub total_shares: u64,
    pub is_active: bool,
    pub bump: u8,
}

pub const VAULT_STATE_SEED: &[u8] = b"my_vault_state";
pub const VAULT_AUTHORITY_SEED: &[u8] = b"my_vault_authority";
```

**Key decisions**:
- Use unique PDA seeds (prefix with your protocol name)
- Add any protocol-specific fields you need
- Always include `is_active` for emergency stops
- Use `Address` instead of `Pubkey` (Quasar uses `Address` for all public keys)

## Step 3: Implement the Three Instructions (2-3 hours)

### 3a. Deposit

```rust
use quasar_lang::prelude::*;
use quasar_spl::token::{self, TokenTransfer};
use yield_adapter_trait::{DepositEvent, YieldAdapterError};
use crate::state::{MyVaultState, VAULT_STATE_SEED};

#[instruction(discriminator = 1)]
pub fn deposit(ctx: Ctx<DepositAccounts>, amount: u64) -> Result<()> {
    require!(amount > 0, YieldAdapterError::ZeroDepositAmount);

    let vault = &mut ctx.accounts.vault_state;
    let clock = quasar_lang::sysvars::clock::Clock::get()?;

    // Calculate shares using the standard formula
    let shares = if vault.total_shares == 0 {
        amount // 1:1 for first deposit
    } else {
        (amount as u128)
            .checked_mul(vault.total_shares as u128)
            .ok_or(YieldAdapterError::ArithmeticOverflow)?
            .checked_div(vault.total_underlying as u128)
            .ok_or(YieldAdapterError::ArithmeticOverflow)? as u64
    };

    // Transfer tokens
    token::transfer(
        ctx.accounts.user_token_account,
        ctx.accounts.vault_token_account,
        ctx.accounts.user,
        amount,
    )?;

    // Update state
    vault.total_underlying = vault.total_underlying
        .checked_add(amount)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;
    vault.total_shares = vault.total_shares
        .checked_add(shares)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;

    // Emit standard event (REQUIRED)
    emit!(DepositEvent {
        user: ctx.accounts.user.address(),
        adapter: crate::ID,
        amount,
        receipt_amount: shares,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
```

### 3b. Withdraw

```rust
#[instruction(discriminator = 2)]
pub fn withdraw(ctx: Ctx<WithdrawAccounts>, amount: u64) -> Result<()> {
    // Similar pattern: burn shares → calculate underlying → transfer out via PDA signer
}
```

Use `token::transfer_signed()` with your vault authority PDA seeds for the outgoing transfer.

### 3c. Current Value

```rust
#[instruction(discriminator = 3)]
pub fn current_value(ctx: Ctx<CurrentValueAccounts>) -> Result<()> {
    let vault = &ctx.accounts.vault_state;
    let clock = quasar_lang::sysvars::clock::Clock::get()?;

    let value = if vault.total_shares == 0 {
        0
    } else {
        let user_shares = ctx.accounts.user_position.shares;
        (user_shares as u128)
            .checked_mul(vault.total_underlying as u128)
            .ok_or(YieldAdapterError::ArithmeticOverflow)?
            .checked_div(vault.total_shares as u128)
            .ok_or(YieldAdapterError::ArithmeticOverflow)? as u64
    };

    emit!(CurrentValueEvent {
        user: ctx.accounts.user.address(),
        adapter: crate::ID,
        value,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
```

Read-only query that emits the user's position value in underlying token units.

## Step 4: Add Protocol-Specific CPI (1-2 hours)

This is where your adapter gets interesting. Instead of holding tokens in a local vault, you'll CPI into the target protocol.

### Example: CPI to a Lending Protocol

```rust
// In your deposit handler, AFTER the token transfer:
// CPI to the protocol's deposit instruction
let cpi_accounts = ProtocolDeposit {
    user_account: ctx.accounts.protocol_user,
    pool: ctx.accounts.protocol_pool,
    vault: ctx.accounts.protocol_vault,
};
let cpi_ctx = CpiContext::new(
    ctx.accounts.protocol_program,
    cpi_accounts,
);
protocol::deposit(cpi_ctx, amount)?;
```

### Tips:
- Study the target protocol's IDL for account requirements
- Clone the protocol's program in your test validator for testing
- Handle protocol-specific errors in the `7000+` error range

## Step 5: Write Tests (1-2 hours)

### Rust QuasarSVM Tests (recommended)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quasar_lang::svm::prelude::*;

    #[test]
    fn deposit_current_value_withdraw() {
        let mut svm = SVM::new();
        let (mut vault, user) = svm.setup();

        // Deposit
        let result = svm.process_instruction(
            deposit::instruction(1_000_000),
            vec![&user],
        );
        assert!(result.is_ok());

        // Current value
        let result = svm.process_instruction(
            current_value::instruction(),
            vec![&user],
        );
        assert!(result.is_ok());

        // Withdraw
        let result = svm.process_instruction(
            withdraw::instruction(500_000),
            vec![&user],
        );
        assert!(result.is_ok());
    }
}
```

### TypeScript Tests (fork parity)

```typescript
describe("my-yield-adapter", () => {
  it("deposits and receives proportional shares", async () => {
    // Setup: create mint, token accounts, initialize vault
    // Action: deposit 1000 USDC
    // Assert: vault.total_underlying == 1000
  });

  it("withdraws proportionally", async () => {
    // Deposit 1000, withdraw 500 shares
    // Assert: user receives 500 USDC
  });

  it("reports correct share price", async () => {
    // Deposit, then query current_value
    // Assert: value matches expected
  });

  it("rejects zero deposits", async () => {
    // Assert: deposit(0) throws ZeroDepositAmount
  });
});
```

## Step 6: Register Your Adapter (10 min)

Once your adapter is deployed, register it with the on-chain registry:

```typescript
import { buildProposeAdapter, buildApproveAdapter } from "quasar-client";

// 1. Propose your adapter
const proposeIx = buildProposeAdapter(
  registryProgramId,
  {
    proposer: wallet.publicKey,
    registryState: registryStatePda,
    adapterEntry: adapterEntryPda,
    adapterProgram: myAdapterProgramId,
    underlyingMint: usdcMint,
    systemProgram: SystemProgram.programId,
  },
  "My Yield Adapter"
);
await sendAndConfirmTransaction(connection, proposeIx, [wallet]);

// 2. Wait for governance approval
// The registry authority will call approve_adapter()
```

## Checklist

Before submitting your adapter:

- [ ] Implements `deposit`, `withdraw`, `current_value`
- [ ] Emits `DepositEvent`, `WithdrawEvent`, `CurrentValueEvent`
- [ ] Uses `checked_*` arithmetic everywhere
- [ ] Validates `amount > 0` on deposit and withdraw
- [ ] Validates `is_active` on state-modifying instructions
- [ ] Validates token mint matches `underlying_mint`
- [ ] Uses PDA authority for vault transfers
- [ ] Has comprehensive tests (deposit, withdraw, current_value, edge cases)
- [ ] Runs `cargo clippy` with zero warnings
- [ ] Protocol-specific errors use error codes 7000+
- [ ] Passes `cargo test` (QuasarSVM tests) and `npm test` (TS tests)
- [ ] Passes mainnet-fork integration tests

## Common Pitfalls

| Pitfall | Solution |
|---|---|
| Unchecked arithmetic | Always use `checked_add`, `checked_sub`, `checked_mul`, `checked_div` |
| Missing event emissions | Every state change MUST emit the corresponding standard event |
| Hardcoded share price | Use the dynamic formula: `total_underlying * shares_burned / total_shares` |
| Missing mint validation | Always validate `token_account.mint == vault.underlying_mint` |
| External signer for vault | Use a PDA derived from known seeds — never an external keypair |
| Wrong discriminator | Each instruction needs a unique single-byte discriminator |
| Using `Pubkey` instead of `Address` | Quasar uses `Address` — import from `quasar_lang::prelude::*` |

## Reference Files

Review these files from the reference adapters:

| What | Where |
|------|-------|
| Vault state | `programs/adapter-kamino/src/state.rs` |
| Instruction handlers | `programs/adapter-kamino/src/lib.rs` |
| QuasarSVM tests | `programs/adapter-kamino/src/tests.rs` |
| TS integration tests | `tests/adapters/kamino.test.ts` |
| Quasar config | `programs/adapter-kamino/Quasar.toml` |

## Need Help?

- Review the [Adapter Standard Specification](./ADAPTER_STANDARD.md)
- Study the five reference adapters in `programs/adapter-*`
- Open an issue on GitHub
