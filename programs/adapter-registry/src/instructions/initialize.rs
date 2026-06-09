use {
    crate::state::{RegistryState, RegistryStateInner},
    quasar_lang::prelude::*,
};

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        mut,
        init,
        payer = authority,
        address = RegistryState::seeds()
    )]
    pub registry_state: Account<RegistryState>,
    pub rent: Sysvar<Rent>,
    pub system_program: Program<SystemProgram>,
}

impl Initialize {
    pub fn handler(&mut self, bumps: &InitializeBumps) -> Result<(), ProgramError> {
        self.registry_state.set_inner(RegistryStateInner {
            authority: *self.authority.address(),
            pending_authority: None,
            total_proposed: 0,
            total_approved: 0,
            bump: bumps.registry_state,
        });
        Ok(())
    }
}
