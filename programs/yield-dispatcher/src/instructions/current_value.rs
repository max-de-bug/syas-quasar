use {
    crate::{
        adapter_cpi::{
            cpi_current_value, read_position_receipt, read_vault_totals, AdapterCurrentValueAccounts,
        },
        adapter_validation::{self, verify_adapter_entry},
        error::DispatcherError,
        events::DispatchCurrentValueEvent,
        state::{DispatcherState, UserPosition},
    },
    quasar_lang::prelude::*,
    yield_adapter_trait::{user_position_underlying_value, REGISTRY_PROGRAM_ID},
};

#[derive(Accounts)]
pub struct CurrentValue {
    pub user: Signer,
    #[account(address = DispatcherState::seeds())]
    pub dispatcher_state: Account<DispatcherState>,
    #[account(
        address = UserPosition::seeds(user.address(), adapter_program.address()),
        constraints(user_position.owner == *user.address()) @ DispatcherError::Unauthorized,
    )]
    pub user_position: Account<UserPosition>,
    #[account(
        constraints(*registry_program.address() == dispatcher_state.registry_program_id)
            @ DispatcherError::RegistryMismatch,
        constraints(*registry_program.address() == REGISTRY_PROGRAM_ID)
            @ DispatcherError::RegistryMismatch,
    )]
    pub registry_program: UncheckedAccount,
    pub adapter_entry: UncheckedAccount,
    pub adapter_program: UncheckedAccount,
    #[account(
        mut,
        constraints(adapter_validation::is_adapter_vault_state(
            adapter_vault_state.to_account_view(),
            adapter_program.address(),
        )) @ DispatcherError::AdapterCpiError,
    )]
    pub adapter_vault_state: UncheckedAccount,
    #[account(
        constraints(adapter_validation::is_adapter_user_position(
            adapter_user_position.to_account_view(),
            adapter_program.address(),
            user.address(),
        )) @ DispatcherError::AdapterCpiError,
    )]
    pub adapter_user_position: UncheckedAccount,
}

impl CurrentValue {
    pub fn handler(&mut self) -> Result<(), ProgramError> {
        verify_adapter_entry(
            self.adapter_entry.to_account_view(),
            self.adapter_program.address(),
        )?;

        cpi_current_value(AdapterCurrentValueAccounts {
            adapter_program: self.adapter_program.to_account_view(),
            user: self.user.to_account_view(),
            vault_state: self.adapter_vault_state.to_account_view(),
            user_position: self.adapter_user_position.to_account_view(),
        })?;

        let (total_underlying, total_shares) =
            read_vault_totals(self.adapter_vault_state.to_account_view())?;
        let adapter_receipt =
            read_position_receipt(self.adapter_user_position.to_account_view())?;
        let value = user_position_underlying_value(
            adapter_receipt,
            total_underlying,
            total_shares,
        )?;

        let clock = <Clock as quasar_lang::sysvars::Sysvar>::get()?;
        let now = clock.unix_timestamp.get();

        emit!(DispatchCurrentValueEvent {
            user: *self.user.address(),
            adapter_program_id: *self.adapter_program.address(),
            value,
            timestamp: now,
        });

        Ok(())
    }
}
