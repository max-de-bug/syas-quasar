use {
    crate::{
        error::RegistryError,
        state::{AdapterEntry, AdapterEntryInner, AdapterStatus, RegistryState},
    },
    quasar_lang::prelude::*,
    yield_adapter_trait::{MAX_ADAPTER_NAME_LEN, MAX_METADATA_URI_LEN},
};

#[derive(Accounts)]
pub struct ProposeAdapter {
    #[account(mut)]
    pub proposer: Signer,
    #[account(mut, address = RegistryState::seeds())]
    pub registry_state: Account<RegistryState>,
    #[account(
        mut,
        init,
        payer = proposer,
        address = AdapterEntry::seeds(adapter_program.address()),
    )]
    pub adapter_entry: Account<AdapterEntry>,
    pub adapter_program: UncheckedAccount,
    pub underlying_mint: UncheckedAccount,
    pub rent: Sysvar<Rent>,
    pub system_program: Program<SystemProgram>,
}

impl ProposeAdapter {
    pub fn handler(
        &mut self,
        name: &str,
        metadata_uri: &str,
        bumps: &ProposeAdapterBumps,
    ) -> Result<(), ProgramError> {
        require!(name.len() <= MAX_ADAPTER_NAME_LEN, RegistryError::NameTooLong);
        require!(
            metadata_uri.len() <= MAX_METADATA_URI_LEN,
            RegistryError::UriTooLong
        );

        let clock = <Clock as quasar_lang::sysvars::Sysvar>::get()?;
        let now = clock.unix_timestamp.get();

        let name_bytes = name.as_bytes();
        let meta_bytes = metadata_uri.as_bytes();
        let mut name_buf = [0u8; MAX_ADAPTER_NAME_LEN];
        let mut meta_buf = [0u8; MAX_METADATA_URI_LEN];
        name_buf[..name_bytes.len()].copy_from_slice(name_bytes);
        meta_buf[..meta_bytes.len()].copy_from_slice(meta_bytes);
        let inner = AdapterEntryInner {
            adapter_program_id: *self.adapter_program.address(),
            status: AdapterStatus::Proposed.as_u8(),
            underlying_mint: *self.underlying_mint.address(),
            proposer: *self.proposer.address(),
            proposed_at: now,
            approved_at: 0,
            revoked_at: 0,
            bump: bumps.adapter_entry,
            name_len: name_bytes.len() as u8,
            name_buf,
            metadata_uri_len: meta_bytes.len() as u8,
            metadata_uri_buf: meta_buf,
        };
        {
            let total: u64 = self.registry_state.total_proposed.into();
            self.registry_state.total_proposed = total
                .checked_add(1)
                .ok_or(RegistryError::InvalidStatus)?
                .into();
        }

        self.adapter_entry.set_inner(inner);

        Ok(())
    }
}
