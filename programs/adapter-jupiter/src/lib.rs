#![no_std]
#![allow(dead_code)]

use quasar_lang::prelude::*;

mod instructions;
mod protocol;
mod state;
pub use state::*;

use instructions::*;

#[cfg(test)]
mod tests;

declare_id!("2acqkTDi2VQ4FCZVDB8PeMVLVWnREogE5HA2GxvHdWxu");

#[program]
mod adapter_jupiter {
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
    pub fn withdraw(ctx: Ctx<Withdraw>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.handler(amount, &ctx.bumps)
    }

    #[instruction(discriminator = 3)]
    pub fn current_value(ctx: CtxWithRemaining<CurrentValue>) -> Result<(), ProgramError> {
        ctx.accounts.handler(ctx.remaining_accounts())
    }
}
