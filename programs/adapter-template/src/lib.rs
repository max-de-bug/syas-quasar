//! # Template Adapter (Quasar)
//!
//! Copy this directory to create a new yield adapter for the Solana Yield Adapter Standard.
//! Replace all "template" / "Template" references with your protocol name.
//!
//! ## Step-by-step
//!
//! 1. **Generate a program ID**: `solana-keygen grind --starts-with YOUR_PREFIX`
//! 2. **Update `declare_id!`** below with your new program ID
//! 3. **Set `EXTERNAL_PROGRAM_ID`** to your protocol's Solana program ID
//! 4. **Rename seeds** in `state.rs` (`template_vault_state` → `your_vault_state`)
//! 5. **Implement protocol CPI** in `protocol.rs` with your instruction discriminators
//! 6. **Add adapter to workspace** `Cargo.toml` + build scripts + `Anchor.toml`
//! 7. **Build**: `quasar build` (or `cargo build-sbf` for Anchor compat)
//! 8. **Test**: `cargo test --package adapter-template` (QuasarSVM) + `npx ts-mocha tests/adapters/template.test.ts`

#![no_std]
#![allow(dead_code)]

use quasar_lang::prelude::*;

pub mod instructions;
pub mod protocol;
pub mod state;
pub use state::*;

use instructions::*;

// QuasarSVM integration tests require the generated `adapter_template_client` crate.
// Run `quasar build` in this directory first, then `cargo test` to enable them.
#[cfg(feature = "client")]
#[cfg(test)]
mod tests;

// IMPORTANT: Replace with your own program ID before deploying.
// Generate with: solana-keygen grind --starts-with YOUR_PREFIX
declare_id!("AzGucBSAxRMme758P9WsXqZASqnea7xZqKr7ys6gvCcX");

/// Program ID of the external yield protocol — set this in protocol.rs.

#[program]
mod adapter_template {
    use super::*;

    /// Initialize the vault state with the underlying token mint.
    #[instruction(discriminator = 0)]
    pub fn initialize(
        ctx: Ctx<Initialize>,
        underlying_mint: Address,
    ) -> Result<(), ProgramError> {
        ctx.accounts.handler(underlying_mint, &ctx.bumps)
    }

    /// Deposit underlying tokens into the yield source.
    #[instruction(discriminator = 1)]
    pub fn deposit(
        ctx: CtxWithRemaining<Deposit>,
        amount: u64,
    ) -> Result<(), ProgramError> {
        ctx.accounts
            .handler(amount, &ctx.bumps, ctx.remaining_accounts())
    }

    /// Withdraw receipt tokens, returning underlying to the user.
    #[instruction(discriminator = 2)]
    pub fn withdraw(ctx: Ctx<Withdraw>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.handler(amount, &ctx.bumps)
    }

    /// Query the current value of a user's position.
    #[instruction(discriminator = 3)]
    pub fn current_value(
        ctx: CtxWithRemaining<CurrentValue>,
    ) -> Result<(), ProgramError> {
        ctx.accounts.handler(ctx.remaining_accounts())
    }
}
