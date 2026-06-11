use quasar_lang::{
    cpi::{CpiDynamic, Seed},
    prelude::*,
};
use yield_adapter_trait::YieldAdapterError;

use crate::state::MarginfiVaultState;

pub const MARGINFI_V2_ID: Address =
    address!("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA");

// sha256("global:lending_account_deposit")[..8]
const MARGINFI_DEPOSIT_DISC: [u8; 8] = [0xab, 0x5e, 0xeb, 0x67, 0x52, 0x40, 0xd4, 0x8c];
// sha256("global:lending_account_withdraw")[..8]
const MARGINFI_WITHDRAW_DISC: [u8; 8] = [0x24, 0x48, 0x4a, 0x13, 0xd2, 0xd2, 0xc0, 0xc0];

macro_rules! remaining_view {
    ($ra:expr) => {
        unsafe {
            core::mem::transmute::<&AccountView, &'a AccountView>($ra.as_account_view_unchecked())
        }
    };
}

pub fn on_deposit<'a>(
    vault: &mut MarginfiVaultState,
    amount: u64,
    authority: &'a AccountView,
    authority_bump: u8,
    vault_token: &'a AccountView,
    token_program: &'a AccountView,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let mut iter = remaining.iter();
    let program = match iter.next() {
        Some(Ok(p)) => p,
        _ => return Ok(()),
    };

    let group = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let marginfi_account = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let bank = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let bank_liquidity_vault = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*program.address(), MARGINFI_V2_ID, YieldAdapterError::AdapterProgramMismatch);

    let mut data = [0u8; 16];
    data[..8].copy_from_slice(&MARGINFI_DEPOSIT_DISC);
    data[8..].copy_from_slice(&amount.to_le_bytes());

    let mut cpi = CpiDynamic::<7, 16>::new(program.address());
    cpi.push_account(remaining_view!(group), false, false)?;
    cpi.push_account(remaining_view!(marginfi_account), false, true)?;
    cpi.push_account(authority, true, false)?;
    cpi.push_account(remaining_view!(bank), false, true)?;
    cpi.push_account(vault_token, false, true)?;
    cpi.push_account(remaining_view!(bank_liquidity_vault), false, true)?;
    cpi.push_account(token_program, false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"marginfi_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn on_withdraw<'a>(
    vault: &mut MarginfiVaultState,
    amount: u64,
    authority: &'a AccountView,
    authority_bump: u8,
    vault_token: &'a AccountView,
    token_program: &'a AccountView,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let mut iter = remaining.iter();
    let program = match iter.next() {
        Some(Ok(p)) => p,
        _ => return Ok(()),
    };

    let group = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let marginfi_account = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let bank = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let vault_authority_pda = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let bank_liquidity_vault = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*program.address(), MARGINFI_V2_ID, YieldAdapterError::AdapterProgramMismatch);

    let mut data = [0u8; 17];
    data[..8].copy_from_slice(&MARGINFI_WITHDRAW_DISC);
    data[8..16].copy_from_slice(&amount.to_le_bytes());
    data[16] = 0;

    let mut cpi = CpiDynamic::<8, 17>::new(program.address());
    cpi.push_account(remaining_view!(group), false, false)?;
    cpi.push_account(remaining_view!(marginfi_account), false, true)?;
    cpi.push_account(authority, true, false)?;
    cpi.push_account(remaining_view!(bank), false, true)?;
    cpi.push_account(vault_token, false, true)?;
    cpi.push_account(remaining_view!(vault_authority_pda), false, false)?;
    cpi.push_account(remaining_view!(bank_liquidity_vault), false, true)?;
    cpi.push_account(token_program, false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"marginfi_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn before_value_query<'a>(
    vault: &mut MarginfiVaultState,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    match iter.next() {
        Some(Ok(prog)) => {
            yield_adapter_trait::verify_protocol_program_account(&prog, MARGINFI_V2_ID)
        }
        _ => Ok(()),
    }
}
