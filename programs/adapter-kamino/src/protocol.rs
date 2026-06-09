use quasar_lang::prelude::*;
use yield_adapter_trait::{record_protocol_routing, YieldAdapterError};

use crate::state::KaminoVaultState;

pub const KAMINO_LEND_ID: Address =
    address!("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");

pub fn on_deposit(
    vault: &mut KaminoVaultState,
    amount: u64,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    record_protocol_routing(
        &mut vault.protocol_routed_underlying,
        amount,
        remaining,
        KAMINO_LEND_ID,
    )
}

pub fn before_value_query(
    vault: &mut KaminoVaultState,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    let prog = iter.next().ok_or(YieldAdapterError::ProtocolCpiError)??;
    yield_adapter_trait::verify_protocol_program_account(&prog, KAMINO_LEND_ID)
}
