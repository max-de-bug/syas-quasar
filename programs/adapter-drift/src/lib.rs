#![no_std]
#![allow(dead_code)]

use quasar_lang::prelude::*;

mod error;
mod instructions;
mod protocol;
mod state;
pub use state::*;

use instructions::*;

#[cfg(test)]
mod tests;

declare_id!("CVfb8T9tf9WEeus4mKWsxTehVezeY9TGwYsSc3JmxWYz");

/// 13-day unstaking cooldown in seconds.
pub const UNSTAKE_COOLDOWN_SECONDS: i64 = 13 * 24 * 60 * 60;

#[program]
mod adapter_drift {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(
        ctx: Ctx<Initialize>,
        underlying_mint: Address,
    ) -> Result<(), ProgramError> {
        ctx.accounts.handler(underlying_mint, &ctx.bumps)
    }

    #[instruction(discriminator = 1)]
    pub fn deposit(
        ctx: CtxWithRemaining<Deposit>,
        amount: u64,
    ) -> Result<(), ProgramError> {
        ctx.accounts
            .handler(amount, &ctx.bumps, ctx.remaining_accounts())
    }

    #[instruction(discriminator = 2)]
    pub fn withdraw(
        ctx: CtxWithRemaining<Withdraw>,
        amount: u64,
    ) -> Result<(), ProgramError> {
        ctx.accounts
            .handler(amount, &ctx.bumps, ctx.remaining_accounts())
    }

    #[instruction(discriminator = 3)]
    pub fn current_value(ctx: CtxWithRemaining<CurrentValue>) -> Result<(), ProgramError> {
        ctx.accounts.handler(ctx.remaining_accounts())
    }
}
