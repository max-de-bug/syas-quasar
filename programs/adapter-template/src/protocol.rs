//! # Protocol CPI Layer
//!
//! Each adapter defines three functions that the instruction handlers call:
//!
//! - **`on_deposit`** — Called after user tokens are transferred into the vault.
//!   Use this to route the deposit to the external protocol via CPI.
//!
//! - **`on_withdraw`** — Called before the vault transfers tokens back to the user.
//!   Use this to withdraw from the external protocol via CPI first.
//!
//! - **`before_value_query`** — Called during `current_value` to verify the
//!   protocol program is available on fork. No-op on localnet.
//!
//! ## Conditional CPI Pattern
//!
//! When `remaining_accounts` is non-empty (mainnet fork), the protocol CPI is
//! executed. When empty (localnet), the call is a no-op — only bookkeeping
//! (`protocol_routed_underlying`) is updated. This lets the same `.so` work
//! on both localnet and mainnet fork without recompilation.

use quasar_lang::prelude::*;
use yield_adapter_trait::{record_protocol_routing, YieldAdapterError};

use crate::state::TemplateVaultState;

/// Program ID of the external yield protocol.
/// Set to `Address::default()` for no-op adapters (e.g., syrupUSDC).
pub const EXTERNAL_PROGRAM_ID: Address = address!("11111111111111111111111111111111");

/// Called after user tokens are transferred into the vault.
///
/// ### Customizing
/// 1. Compute the instruction discriminator: `sha256("global:<your_instruction>")[..8]`
/// 2. Build a raw `Instruction` with the correct accounts and data layout
/// 3. Call `invoke_signed` with the vault authority PDA seeds
///
/// See the Kamino or MarginFi adapters for a complete example with `invoke_signed`.
pub fn on_deposit(
    vault: &mut TemplateVaultState,
    amount: u64,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    // On fork (remaining accounts present), execute real protocol CPI.
    // On localnet (remaining accounts absent), just update bookkeeping.
    //
    // Example for a real protocol:
    // ```
    // use quasar_lang::cpi::Seed;
    // if remaining.iter().next().is_some() {
    //     let bump = [vault.bump]; // vault_authority bump
    //     let seeds = [
    //         Seed::from(b"template_vault_authority" as &[u8]),
    //         Seed::from(bump.as_ref()),
    //     ];
    //     // ... build Instruction and call invoke_signed ...
    // }
    // ```
    //
    // For now, use the shared routing helper which validates the protocol
    // program ID is present in remaining accounts and updates the counter.
    record_protocol_routing(
        &mut vault.protocol_routed_underlying,
        amount,
        remaining,
        EXTERNAL_PROGRAM_ID,
    )
}

/// Called before the vault transfers underlying tokens back to the user.
///
/// Same conditional CPI pattern as `on_deposit`. When `EXTERNAL_PROGRAM_ID`
/// is `Address::default()` (no-op), this function does nothing.
pub fn on_withdraw(
    vault: &mut TemplateVaultState,
    _amount: u64,
    _remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    // Track the amount routed through the protocol for auditability.
    // let routed: u64 = vault.protocol_routed_underlying.into();
    // vault.protocol_routed_underlying = routed
    //     .checked_sub(_amount)
    //     .ok_or(YieldAdapterError::ArithmeticOverflow)?
    //     .into();
    let _ = vault;
    Ok(())
}

/// Called before a `current_value` query.
///
/// On fork, validates the remaining accounts contain the protocol program.
/// On localnet, silently succeeds.
pub fn before_value_query(
    vault: &mut TemplateVaultState,
    remaining: RemainingAccounts<'_>,
) -> Result<(), ProgramError> {
    let _ = vault;
    let mut iter = remaining.iter();
    if let Some(prog) = iter.next() {
        let prog = prog?;
        yield_adapter_trait::verify_protocol_program_account(&prog, EXTERNAL_PROGRAM_ID)
    } else {
        Ok(())
    }
}
