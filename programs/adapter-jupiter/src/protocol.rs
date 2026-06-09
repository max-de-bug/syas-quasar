use quasar_lang::prelude::*;
use yield_adapter_trait::{record_protocol_routing, YieldAdapterError};

use crate::state::JupiterVaultState;

pub const JUPITER_PERP_ID: Address =
    address!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu");

pub fn on_deposit(
    vault: &mut JupiterVaultState,
    amount: u64,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    record_protocol_routing(
        &mut vault.protocol_routed_underlying,
        amount,
        remaining,
        JUPITER_PERP_ID,
    )
}

pub fn before_value_query(
    vault: &mut JupiterVaultState,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    let prog = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    yield_adapter_trait::verify_protocol_program_account(&prog, JUPITER_PERP_ID)
}
