use quasar_lang::prelude::*;
use yield_adapter_trait::{record_protocol_routing, YieldAdapterError};

use crate::state::DriftVaultState;

pub const DRIFT_V2_ID: Address =
    address!("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH");

pub fn on_deposit(
    vault: &mut DriftVaultState,
    amount: u64,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    record_protocol_routing(
        &mut vault.protocol_routed_underlying,
        amount,
        remaining,
        DRIFT_V2_ID,
    )
}

pub fn before_value_query(
    vault: &mut DriftVaultState,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    let prog = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    yield_adapter_trait::verify_protocol_program_account(&prog, DRIFT_V2_ID)
}
