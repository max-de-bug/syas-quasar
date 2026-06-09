//! CPI routing from the dispatcher into registered reference adapters.

use {
    crate::error::DispatcherError,
    quasar_lang::{
        cpi::CpiDynamic,
        prelude::*,
    },
    yield_adapter_trait::{
        read_adapter_position_receipt, read_reference_vault_totals, DRIFT_ADAPTER_ID,
        JUPITER_ADAPTER_ID, KAMINO_ADAPTER_ID, MAPLE_ADAPTER_ID, MARGINFI_ADAPTER_ID,
    },
};

const DEPOSIT_DISC: u8 = 1;
const WITHDRAW_DISC: u8 = 2;
const CURRENT_VALUE_DISC: u8 = 3;

fn is_reference_adapter(program: &Address) -> bool {
    matches!(
        *program,
        KAMINO_ADAPTER_ID
            | MARGINFI_ADAPTER_ID
            | JUPITER_ADAPTER_ID
            | MAPLE_ADAPTER_ID
            | DRIFT_ADAPTER_ID
    )
}

fn encode_amount_ix(discriminator: u8, amount: u64) -> Result<[u8; 9], ProgramError> {
    let mut data = [0u8; 9];
    data[0] = discriminator;
    data[1..9].copy_from_slice(&amount.to_le_bytes());
    Ok(data)
}

pub fn read_vault_totals(view: &AccountView) -> Result<(u64, u64), ProgramError> {
    let data = view.try_borrow()?;
    read_reference_vault_totals(&data).map_err(|_| DispatcherError::AdapterCpiError.into())
}

pub struct AdapterDepositAccounts<'a> {
    pub adapter_program: &'a AccountView,
    pub user: &'a AccountView,
    pub vault_state: &'a AccountView,
    pub user_position: &'a AccountView,
    pub user_token_account: &'a AccountView,
    pub vault_token_account: &'a AccountView,
    pub vault_authority: &'a AccountView,
    pub token_program: &'a AccountView,
    pub system_program: &'a AccountView,
}

pub fn cpi_deposit(accounts: AdapterDepositAccounts<'_>, amount: u64) -> Result<u64, ProgramError> {
    if !is_reference_adapter(accounts.adapter_program.address()) {
        return Err(DispatcherError::AdapterNotApproved.into());
    }

    let (_, shares_before) = read_vault_totals(accounts.vault_state)?;

    let mut cpi = CpiDynamic::<8, 16>::new(accounts.adapter_program.address());
    cpi.push_account(accounts.user, true, true)?;
    cpi.push_account(accounts.vault_state, false, true)?;
    cpi.push_account(accounts.user_position, false, true)?;
    cpi.push_account(accounts.user_token_account, false, true)?;
    cpi.push_account(accounts.vault_authority, false, false)?;
    cpi.push_account(accounts.vault_token_account, false, true)?;
    cpi.push_account(accounts.token_program, false, false)?;
    cpi.push_account(accounts.system_program, false, false)?;
    cpi.set_data(&encode_amount_ix(DEPOSIT_DISC, amount)?)?;
    cpi.invoke()?;

    let (_, shares_after) = read_vault_totals(accounts.vault_state)?;
    Ok(shares_after
        .checked_sub(shares_before)
        .ok_or(DispatcherError::AdapterCpiError)?)
}

pub struct AdapterWithdrawAccounts<'a> {
    pub adapter_program: &'a AccountView,
    pub user: &'a AccountView,
    pub vault_state: &'a AccountView,
    pub user_position: &'a AccountView,
    pub user_token_account: &'a AccountView,
    pub vault_token_account: &'a AccountView,
    pub vault_authority: &'a AccountView,
    pub token_program: &'a AccountView,
}

pub fn cpi_withdraw(accounts: AdapterWithdrawAccounts<'_>, shares: u64) -> Result<(), ProgramError> {
    if !is_reference_adapter(accounts.adapter_program.address()) {
        return Err(DispatcherError::AdapterNotApproved.into());
    }

    let mut cpi = CpiDynamic::<7, 16>::new(accounts.adapter_program.address());
    cpi.push_account(accounts.user, true, true)?;
    cpi.push_account(accounts.vault_state, false, true)?;
    cpi.push_account(accounts.user_position, false, true)?;
    cpi.push_account(accounts.user_token_account, false, true)?;
    cpi.push_account(accounts.vault_token_account, false, true)?;
    cpi.push_account(accounts.vault_authority, false, false)?;
    cpi.push_account(accounts.token_program, false, false)?;
    cpi.set_data(&encode_amount_ix(WITHDRAW_DISC, shares)?)?;
    cpi.invoke()
}

pub struct AdapterCurrentValueAccounts<'a> {
    pub adapter_program: &'a AccountView,
    pub user: &'a AccountView,
    pub vault_state: &'a AccountView,
    pub user_position: &'a AccountView,
}

pub fn cpi_current_value(accounts: AdapterCurrentValueAccounts<'_>) -> Result<(), ProgramError> {
    if !is_reference_adapter(accounts.adapter_program.address()) {
        return Err(DispatcherError::AdapterNotApproved.into());
    }

    let mut cpi = CpiDynamic::<4, 4>::new(accounts.adapter_program.address());
    cpi.push_account(accounts.user, true, false)?;
    cpi.push_account(accounts.vault_state, false, true)?;
    cpi.push_account(accounts.user_position, false, false)?;
    cpi.set_data(&[CURRENT_VALUE_DISC])?;
    cpi.invoke()
}

pub fn read_position_receipt(view: &AccountView) -> Result<u64, ProgramError> {
    let data = view.try_borrow()?;
    read_adapter_position_receipt(&data).map_err(|_| DispatcherError::AdapterCpiError.into())
}
