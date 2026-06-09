use quasar_lang::prelude::*;
use yield_adapter_trait::{record_protocol_routing, YieldAdapterError};

use crate::state::MarginfiVaultState;

pub const MARGINFI_V2_ID: Address =
    address!("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA");

pub fn on_deposit(
    vault: &mut MarginfiVaultState,
    amount: u64,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    record_protocol_routing(
        &mut vault.protocol_routed_underlying,
        amount,
        remaining,
        MARGINFI_V2_ID,
    )
}

pub fn before_value_query(
    vault: &mut MarginfiVaultState,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    let prog = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    yield_adapter_trait::verify_protocol_program_account(&prog, MARGINFI_V2_ID)
}
