use {
    crate::{
        adapter_cpi::{cpi_deposit, AdapterDepositAccounts},
        adapter_validation::{self, verify_adapter_entry},
        error::DispatcherError,
        events::DispatchDepositEvent,
        state::{DispatcherState, UserPosition, UserPositionInner},
    },
    quasar_lang::prelude::*,
    quasar_spl::prelude::*,
    yield_adapter_trait::REGISTRY_PROGRAM_ID,
};

#[derive(Accounts)]
pub struct Deposit {
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
        init(idempotent),
        payer = user,
        address = UserPosition::seeds(user.address(), adapter_program.address()),
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
    #[account(
        constraints(adapter_program.to_account_view().executable())
            @ DispatcherError::AdapterNotApproved,
    )]
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
        constraints(adapter_validation::is_adapter_user_position_pda(
            adapter_user_position.to_account_view(),
            adapter_program.address(),
            user.address(),
        )) @ DispatcherError::AdapterCpiError,
    )]
    pub adapter_user_position: UncheckedAccount,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<SystemProgram>,
}

impl Deposit {
    pub fn handler(&mut self, amount: u64, bumps: &DepositBumps) -> Result<(), ProgramError> {
        require!(amount > 0, DispatcherError::ZeroAmount);

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

        let shares_minted = cpi_deposit(
            AdapterDepositAccounts {
                adapter_program: self.adapter_program.to_account_view(),
                user: self.user.to_account_view(),
                vault_state: self.adapter_vault_state.to_account_view(),
                user_position: self.adapter_user_position.to_account_view(),
                user_token_account: self.user_token_account.to_account_view(),
                vault_token_account: self.adapter_vault.to_account_view(),
                vault_authority: self.adapter_vault_authority.to_account_view(),
                token_program: self.token_program.to_account_view(),
                system_program: self.system_program.to_account_view(),
            },
            amount,
        )?;

        let clock = <Clock as quasar_lang::sysvars::Sysvar>::get()?;
        let now = clock.unix_timestamp.get();

        if self.user_position.owner == Address::default() {
            self.user_position.set_inner(UserPositionInner {
                owner: *self.user.address(),
                adapter_program_id: *self.adapter_program.address(),
                deposited_amount: 0,
                withdrawn_amount: 0,
                receipt_token_balance: 0,
                last_updated: 0,
                bump: bumps.user_position,
            });
        }

        {
            let deposited: u64 = self.user_position.deposited_amount.into();
            self.user_position.deposited_amount = deposited
                .checked_add(amount)
                .ok_or(DispatcherError::AdapterCpiError)?
                .into();
        }
        {
            let receipt: u64 = self.user_position.receipt_token_balance.into();
            self.user_position.receipt_token_balance = receipt
                .checked_add(shares_minted)
                .ok_or(DispatcherError::AdapterCpiError)?
                .into();
        }
        self.user_position.last_updated = now.into();

        {
            let total: u64 = self.dispatcher_state.total_deposits.into();
            self.dispatcher_state.total_deposits = total
                .checked_add(1)
                .ok_or(DispatcherError::AdapterCpiError)?
                .into();
        }

        emit!(DispatchDepositEvent {
            user: *self.user.address(),
            adapter_program_id: *self.adapter_program.address(),
            amount,
            timestamp: now,
        });

        Ok(())
    }
}
