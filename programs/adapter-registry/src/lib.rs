#![no_std]
#![allow(dead_code)]

use quasar_lang::prelude::*;

mod error;
mod instructions;
mod state;
pub use state::*;

use instructions::*;

#[cfg(test)]
mod tests;

declare_id!("CeyDkRgegNUz2TeFfFjRdL89G9EGGDymiqHoJkeFGcZ4");

#[program]
mod adapter_registry {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(ctx: Ctx<Initialize>) -> Result<(), ProgramError> {
        ctx.accounts.handler(&ctx.bumps)
    }

    #[instruction(discriminator = 1)]
    pub fn propose_adapter(
        ctx: Ctx<ProposeAdapter>,
        #[max(32)] name: &str,
        #[max(200)] metadata_uri: &str,
    ) -> Result<(), ProgramError> {
        ctx.accounts.handler(name, metadata_uri, &ctx.bumps)
    }

    #[instruction(discriminator = 2)]
    pub fn approve_adapter(ctx: Ctx<ApproveAdapter>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }

    #[instruction(discriminator = 3)]
    pub fn revoke_adapter(ctx: Ctx<RevokeAdapter>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }

    #[instruction(discriminator = 4)]
    pub fn transfer_governance(ctx: Ctx<TransferGovernance>) -> Result<(), ProgramError> {
        ctx.accounts.handler()
    }
}
