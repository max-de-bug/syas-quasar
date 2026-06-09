use {
    crate::{error::RegistryError, state::RegistryState},
    quasar_lang::prelude::*,
};

#[derive(Accounts)]
pub struct TransferGovernance {
    pub authority: Signer,
    #[account(
        mut,
        address = RegistryState::seeds(),
        constraints(registry_state.authority == *authority.address()) @ RegistryError::Unauthorized,
    )]
    pub registry_state: Account<RegistryState>,
    pub new_authority: UncheckedAccount,
}

impl TransferGovernance {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        self.registry_state.authority = *self.new_authority.address();
        Ok(())
    }
}
