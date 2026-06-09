use {
    crate::{
        adapter_cpi::{cpi_withdraw, AdapterWithdrawAccounts},
        adapter_validation::{self, verify_adapter_entry},
        error::DispatcherError,
        events::DispatchWithdrawEvent,
        state::{DispatcherState, UserPosition},
    },
    quasar_lang::prelude::*,
    quasar_spl::prelude::*,
    yield_adapter_trait::REGISTRY_PROGRAM_ID,
};

#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut)]
    pub user: Signer,
    #[account(
        mut,
        address = DispatcherState::seeds(),
        constraints(!Into::<bool>::into(dispatcher_state.is_paused)) @ DispatcherError::DispatcherPaused,
    )]
    pub dispatcher_state: Account<DispatcherState>,
    #[account(
        mut,
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
    #[account(mut)]
    pub user_token_account: Account<Token>,
    #[account(
        mut,
        constraints(adapter_validation::is_adapter_vault_state(
            adapter_vault_state.to_account_view(),
            adapter_program.address(),
        )) @ DispatcherError::AdapterCpiError,
    )]
    pub adapter_vault_state: UncheckedAccount,
    #[account(mut)]
    pub adapter_vault: Account<Token>,
    #[account(
        constraints(adapter_validation::is_adapter_vault_authority(
            adapter_vault_authority.to_account_view(),
            adapter_program.address(),
        )) @ DispatcherError::AdapterCpiError,
    )]
    pub adapter_vault_authority: UncheckedAccount,
    #[account(
        mut,
        constraints(adapter_validation::is_adapter_user_position(
            adapter_user_position.to_account_view(),
            adapter_program.address(),
            user.address(),
        )) @ DispatcherError::AdapterCpiError,
    )]
    pub adapter_user_position: UncheckedAccount,
    pub token_program: Program<TokenProgram>,
}

impl Withdraw {
    pub fn handler(&mut self, shares: u64) -> Result<(), ProgramError> {
        require!(shares > 0, DispatcherError::ZeroAmount);
        require!(
            self.user_position.receipt_token_balance >= shares,
            DispatcherError::AdapterCpiError
        );

        verify_adapter_entry(
            self.adapter_entry.to_account_view(),
            self.adapter_program.address(),
        )?;
        let underlying_mint =
            adapter_validation::adapter_entry_underlying_mint(self.adapter_entry.to_account_view())?;
        require_keys_eq!(
            *self.user_token_account.mint(),
            underlying_mint,
            DispatcherError::AdapterCpiError
        );
        require_keys_eq!(
            *self.adapter_vault.mint(),
            underlying_mint,
            DispatcherError::AdapterCpiError
        );
        require_keys_eq!(
            *self.adapter_vault.owner(),
            *self.adapter_vault_authority.address(),
            DispatcherError::AdapterCpiError
        );

        cpi_withdraw(
            AdapterWithdrawAccounts {
                adapter_program: self.adapter_program.to_account_view(),
                user: self.user.to_account_view(),
                vault_state: self.adapter_vault_state.to_account_view(),
                user_position: self.adapter_user_position.to_account_view(),
                user_token_account: self.user_token_account.to_account_view(),
                vault_token_account: self.adapter_vault.to_account_view(),
                vault_authority: self.adapter_vault_authority.to_account_view(),
                token_program: self.token_program.to_account_view(),
            },
            shares,
        )?;

        let clock = <Clock as quasar_lang::sysvars::Sysvar>::get()?;
        let now = clock.unix_timestamp.get();

        {
            let withdrawn: u64 = self.user_position.withdrawn_amount.into();
            self.user_position.withdrawn_amount = withdrawn
                .checked_add(shares)
                .ok_or(DispatcherError::AdapterCpiError)?
                .into();
        }
        {
            let receipt: u64 = self.user_position.receipt_token_balance.into();
            self.user_position.receipt_token_balance = receipt
                .checked_sub(shares)
                .ok_or(DispatcherError::AdapterCpiError)?
                .into();
        }
        self.user_position.last_updated = now.into();

        {
            let total: u64 = self.dispatcher_state.total_withdrawals.into();
            self.dispatcher_state.total_withdrawals = total
                .checked_add(1)
                .ok_or(DispatcherError::AdapterCpiError)?
                .into();
        }

        emit!(DispatchWithdrawEvent {
            user: *self.user.address(),
            adapter_program_id: *self.adapter_program.address(),
            amount: shares,
            timestamp: now,
        });

        Ok(())
    }
}
