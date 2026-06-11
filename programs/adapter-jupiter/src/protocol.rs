use quasar_lang::{
    cpi::{CpiDynamic, Seed},
    prelude::*,
};
use yield_adapter_trait::YieldAdapterError;

use crate::state::JupiterVaultState;

pub const JUPITER_PERP_ID: Address =
    address!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu");

// sha256("global:add_liquidity2")[..8]
const ADD_LIQUIDITY_DISC: [u8; 8] = [0xe4, 0xa2, 0x4e, 0x1c, 0x46, 0xdb, 0x74, 0x73];
// sha256("global:remove_liquidity2")[..8]
const REMOVE_LIQUIDITY_DISC: [u8; 8] = [0xe6, 0xd7, 0x52, 0x7f, 0xf1, 0x65, 0xe3, 0x92];

macro_rules! remaining_view {
    ($ra:expr) => {
        unsafe {
            core::mem::transmute::<&AccountView, &'a AccountView>($ra.as_account_view_unchecked())
        }
    };
}

pub fn on_deposit<'a>(
    vault: &mut JupiterVaultState,
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
    let transfer_authority = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let perpetuals = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let pool = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody_doves_price = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody_pyth_price = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody_token_account = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let lp_token_mint = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let event_authority = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*program.address(), JUPITER_PERP_ID, YieldAdapterError::AdapterProgramMismatch);

    // add_liquidity2 has no data args — amount is controlled by transfer_authority approval
    let data = ADD_LIQUIDITY_DISC;

    let mut cpi = CpiDynamic::<14, 8>::new(program.address());
    cpi.push_account(authority, true, false)?;
    cpi.push_account(vault_token, false, true)?;
    let vault_lp = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    cpi.push_account(remaining_view!(vault_lp), false, true)?;
    cpi.push_account(remaining_view!(transfer_authority), false, false)?;
    cpi.push_account(remaining_view!(perpetuals), false, false)?;
    cpi.push_account(remaining_view!(pool), false, true)?;
    cpi.push_account(remaining_view!(custody), false, true)?;
    cpi.push_account(remaining_view!(custody_doves_price), false, false)?;
    cpi.push_account(remaining_view!(custody_pyth_price), false, false)?;
    cpi.push_account(remaining_view!(custody_token_account), false, true)?;
    cpi.push_account(remaining_view!(lp_token_mint), false, true)?;
    cpi.push_account(token_program, false, false)?;
    cpi.push_account(remaining_view!(event_authority), false, false)?;
    cpi.push_account(remaining_view!(program), false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"jupiter_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn on_withdraw<'a>(
    vault: &mut JupiterVaultState,
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
    let transfer_authority = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let perpetuals = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let pool = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody_doves_price = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody_pyth_price = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let custody_token_account = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let lp_token_mint = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let event_authority = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*program.address(), JUPITER_PERP_ID, YieldAdapterError::AdapterProgramMismatch);

    let data = REMOVE_LIQUIDITY_DISC;

    let mut cpi = CpiDynamic::<14, 8>::new(program.address());
    cpi.push_account(authority, true, false)?;
    cpi.push_account(vault_token, false, true)?;
    let vault_lp = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    cpi.push_account(remaining_view!(vault_lp), false, true)?;
    cpi.push_account(remaining_view!(transfer_authority), false, false)?;
    cpi.push_account(remaining_view!(perpetuals), false, false)?;
    cpi.push_account(remaining_view!(pool), false, true)?;
    cpi.push_account(remaining_view!(custody), false, true)?;
    cpi.push_account(remaining_view!(custody_doves_price), false, false)?;
    cpi.push_account(remaining_view!(custody_pyth_price), false, false)?;
    cpi.push_account(remaining_view!(custody_token_account), false, true)?;
    cpi.push_account(remaining_view!(lp_token_mint), false, true)?;
    cpi.push_account(token_program, false, false)?;
    cpi.push_account(remaining_view!(event_authority), false, false)?;
    cpi.push_account(remaining_view!(program), false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"jupiter_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn before_value_query<'a>(
    vault: &mut JupiterVaultState,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    match iter.next() {
        Some(Ok(prog)) => {
            yield_adapter_trait::verify_protocol_program_account(&prog, JUPITER_PERP_ID)
        }
        _ => Ok(()),
    }
}
