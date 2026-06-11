use quasar_lang::{
    cpi::{CpiDynamic, Seed},
    prelude::*,
};
use crate::state::DriftVaultState;
use yield_adapter_trait::YieldAdapterError;

pub const DRIFT_V2_ID: Address =
    address!("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH");

// Drift v2 uses instruction tags for spot market operations
const DRIFT_SPOT_DEPOSIT_TAG: u8 = 15;
const DRIFT_SPOT_WITHDRAW_TAG: u8 = 16;

macro_rules! remaining_view {
    ($ra:expr) => {
        unsafe {
            core::mem::transmute::<&AccountView, &'a AccountView>($ra.as_account_view_unchecked())
        }
    };
}

pub fn on_deposit<'a>(
    vault: &mut DriftVaultState,
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
    let state = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let user = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let spot_market = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let spot_market_vault = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let drift_signer = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let sysvar_ix = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*program.address(), DRIFT_V2_ID, YieldAdapterError::AdapterProgramMismatch);

    // Drift deposit_into_spot_market instruction data:
    // u8 tag + u16 market_index + u64 amount + bool reduce_only
    let mut data = [0u8; 12];
    data[0] = DRIFT_SPOT_DEPOSIT_TAG;
    data[2..4].copy_from_slice(&0u16.to_le_bytes());
    data[4..12].copy_from_slice(&amount.to_le_bytes());

    let mut cpi = CpiDynamic::<8, 12>::new(program.address());
    cpi.push_account(authority, true, false)?;
    cpi.push_account(remaining_view!(state), false, false)?;
    cpi.push_account(remaining_view!(user), false, true)?;
    cpi.push_account(remaining_view!(spot_market), false, true)?;
    cpi.push_account(remaining_view!(spot_market_vault), false, true)?;
    cpi.push_account(vault_token, false, true)?;
    cpi.push_account(remaining_view!(drift_signer), false, false)?;
    cpi.push_account(token_program, false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"drift_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn on_withdraw<'a>(
    vault: &mut DriftVaultState,
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
    let state = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let user = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let spot_market = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let spot_market_vault = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let drift_signer = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    let sysvar_ix = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;

    require_keys_eq!(*program.address(), DRIFT_V2_ID, YieldAdapterError::AdapterProgramMismatch);

    // Drift withdraw_from_spot_market instruction data:
    // u8 tag + u16 market_index + u64 amount + bool reduce_only
    let mut data = [0u8; 12];
    data[0] = DRIFT_SPOT_WITHDRAW_TAG;
    data[2..4].copy_from_slice(&0u16.to_le_bytes());
    data[4..12].copy_from_slice(&amount.to_le_bytes());

    let mut cpi = CpiDynamic::<8, 12>::new(program.address());
    cpi.push_account(authority, true, false)?;
    cpi.push_account(remaining_view!(state), false, false)?;
    cpi.push_account(remaining_view!(user), false, true)?;
    cpi.push_account(remaining_view!(spot_market), false, true)?;
    cpi.push_account(remaining_view!(spot_market_vault), false, true)?;
    cpi.push_account(vault_token, false, true)?;
    cpi.push_account(remaining_view!(drift_signer), false, false)?;
    cpi.push_account(token_program, false, false)?;
    cpi.set_data(&data)?;

    let bump = [authority_bump];
    let seeds = [
        Seed::from(b"drift_vault_authority" as &[u8]),
        Seed::from(bump.as_ref()),
    ];
    cpi.invoke_signed(&seeds)?;

    Ok(())
}

pub fn before_value_query<'a>(
    vault: &mut DriftVaultState,
    remaining: RemainingAccounts<'a>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    match iter.next() {
        Some(Ok(prog)) => {
            yield_adapter_trait::verify_protocol_program_account(&prog, DRIFT_V2_ID)
        }
        _ => Ok(()),
    }
}
