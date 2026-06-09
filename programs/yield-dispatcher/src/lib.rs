#![no_std]
#![allow(dead_code)]

use quasar_lang::prelude::*;

mod adapter_cpi;
mod adapter_validation;
mod error;
mod events;
mod instructions;
mod state;

use instructions::*;

#[cfg(test)]
mod tests;

declare_id!("7oUKys5XKMzD2NmFCZyLDyTF2Hm1VH3qX8jVfZEY4f3r");

#[program]
mod yield_dispatcher {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(ctx: Ctx<Initialize>) -> Result<(), ProgramError> {
        ctx.accounts.handler(&ctx.bumps)
    }

    #[instruction(discriminator = 1)]
    pub fn deposit(ctx: Ctx<Deposit>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.handler(amount, &ctx.bumps)
    }

    #[instruction(discriminator = 2)]
    pub fn withdraw(ctx: Ctx<Withdraw>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.handler(amount)
    }

    #[instruction(discriminator = 3)]
    pub fn current_value(ctx: Ctx<CurrentValue>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }
}
