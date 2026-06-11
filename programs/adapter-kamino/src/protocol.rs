use quasar_lang::{
    cpi::{CpiDynamic, Seed},
    prelude::*,
};
use yield_adapter_trait::YieldAdapterError;

use crate::state::KaminoVaultState;

pub const KAMINO_LEND_ID: Address =
    address!("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");

// sha256("global:deposit_reserve_liquidity")[..8]
const KAMINO_DEPOSIT_DISC: [u8; 8] = [0xa9, 0xc9, 0x1e, 0x7e, 0x06, 0xcd, 0x66, 0x44];
// sha256("global:redeem_reserve_collateral")[..8]
const KAMINO_REDEEM_DISC: [u8; 8] = [0xea, 0x75, 0xb5, 0x7d, 0xb9, 0x8e, 0xdc, 0x1d];

macro_rules! remaining_view {
    ($ra:expr) => {
        unsafe {
            core::mem::transmute::<&AccountView, &'a AccountView>($ra.as_account_view_unchecked())
        }
    };
}

pub fn on_deposit<'a>(
    vault: &mut KaminoVaultState,
    amount: u64,
    authority: &'a AccountView,
    authority_bump: u8,
    vault_token: &'a AccountView,
    token_program: &'a AccountView,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let mut iter = remaining.iter();
    let kamino_prog = match iter.next() {
        Some(Ok(p)) => p,
        _ => return Ok(()),
    };

    let reserve = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let lending_market = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let market_authority = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let liq_mint = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let liq_supply = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let collat_mint = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let vault_ctoken = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let collat_token_prog = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let sysvar_ix = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*kamino_prog.address(), KAMINO_LEND_ID, YieldAdapterError::AdapterProgramMismatch);

    let mut data = [0u8; 16];
    data[..8].copy_from_slice(&KAMINO_DEPOSIT_DISC);
    data[8..].copy_from_slice(&amount.to_le_bytes());

    let mut cpi = CpiDynamic::<12, 16>::new(kamino_prog.address());
    cpi.push_account(authority, true, false)?;
    cpi.push_account(remaining_view!(reserve), false, true)?;
    cpi.push_account(remaining_view!(lending_market), false, false)?;
    cpi.push_account(remaining_view!(market_authority), false, false)?;
    cpi.push_account(remaining_view!(liq_mint), false, false)?;
    cpi.push_account(remaining_view!(liq_supply), false, true)?;
    cpi.push_account(vault_token, false, true)?;
    cpi.push_account(remaining_view!(collat_mint), false, false)?;
    cpi.push_account(remaining_view!(vault_ctoken), false, true)?;
    cpi.push_account(remaining_view!(collat_token_prog), false, false)?;
    cpi.push_account(token_program, false, false)?;
    cpi.push_account(remaining_view!(sysvar_ix), false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"kamino_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn on_withdraw<'a>(
    vault: &mut KaminoVaultState,
    amount: u64,
    authority: &'a AccountView,
    authority_bump: u8,
    vault_token: &'a AccountView,
    token_program: &'a AccountView,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let mut iter = remaining.iter();
    let kamino_prog = match iter.next() {
        Some(Ok(p)) => p,
        _ => return Ok(()),
    };

    let reserve = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let lending_market = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let market_authority = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let liq_mint = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let collat_mint = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let liq_supply = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let vault_ctoken = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let collat_token_prog = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let sysvar_ix = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*kamino_prog.address(), KAMINO_LEND_ID, YieldAdapterError::AdapterProgramMismatch);

    let mut data = [0u8; 16];
    data[..8].copy_from_slice(&KAMINO_REDEEM_DISC);
    data[8..].copy_from_slice(&amount.to_le_bytes());

    let mut cpi = CpiDynamic::<12, 16>::new(kamino_prog.address());
    cpi.push_account(authority, true, false)?;
    cpi.push_account(remaining_view!(lending_market), false, false)?;
    cpi.push_account(remaining_view!(reserve), false, true)?;
    cpi.push_account(remaining_view!(market_authority), false, false)?;
    cpi.push_account(remaining_view!(liq_mint), false, false)?;
    cpi.push_account(remaining_view!(collat_mint), false, false)?;
    cpi.push_account(remaining_view!(liq_supply), false, true)?;
    cpi.push_account(remaining_view!(vault_ctoken), false, true)?;
    cpi.push_account(vault_token, false, true)?;
    cpi.push_account(remaining_view!(collat_token_prog), false, false)?;
    cpi.push_account(token_program, false, false)?;
    cpi.push_account(remaining_view!(sysvar_ix), false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"kamino_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn before_value_query<'a>(
    vault: &mut KaminoVaultState,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    match iter.next() {
        Some(Ok(prog)) => {
            yield_adapter_trait::verify_protocol_program_account(&prog, KAMINO_LEND_ID)
        }
        _ => Ok(()),
    }
}
