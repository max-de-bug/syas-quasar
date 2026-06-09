use {
    crate::{
        error::RegistryError,
        state::{AdapterEntry, AdapterStatus, RegistryState},
    },
    quasar_lang::prelude::*,
};

#[derive(Accounts)]
pub struct ApproveAdapter {
    pub authority: Signer,
    #[account(
        mut,
        address = RegistryState::seeds(),
        constraints(registry_state.authority == *authority.address()) @ RegistryError::Unauthorized,
    )]
    pub registry_state: Account<RegistryState>,
    #[account(
        mut,
        constraints(adapter_entry.status == AdapterStatus::Proposed.as_u8()) @ RegistryError::InvalidStatus,
    )]
    pub adapter_entry: Account<AdapterEntry>,
}

impl ApproveAdapter {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        let seeds = AdapterEntry::seeds(&self.adapter_entry.adapter_program_id);
        let bump = seeds
            .verify(self.adapter_entry.address(), &crate::ID)
            .map_err(|_| RegistryError::InvalidStatus)?;
        require_eq!(self.adapter_entry.bump, bump, RegistryError::InvalidStatus);

        let clock = <Clock as quasar_lang::sysvars::Sysvar>::get()?;
        let now = clock.unix_timestamp.get();

        self.adapter_entry.status = AdapterStatus::Approved.as_u8();
        self.adapter_entry.approved_at = now.into();

        {
            let total: u64 = self.registry_state.total_approved.into();
            self.registry_state.total_approved = total
                .checked_add(1)
                .ok_or(RegistryError::InvalidStatus)?
                .into();
        }

        Ok(())
    }
}
