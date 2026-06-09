//! Solana Yield Adapter Standard — shared trait definitions (Quasar).

#![no_std]

use quasar_lang::prelude::*;

pub const ADAPTER_POSITION_SEED: &[u8] = b"adapter_position";
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";
pub const ADAPTER_METADATA_SEED: &[u8] = b"adapter_metadata";
pub const MAX_ADAPTER_NAME_LEN: usize = 32;
pub const MAX_METADATA_URI_LEN: usize = 200;
pub const ADAPTER_STANDARD_VERSION: u8 = 1;

pub const KAMINO_ADAPTER_ID: Address =
    address!("BzuVWb3UgCW6axee6ZNb812D268XrWkJsE7mxkX9b3Kp");
pub const MARGINFI_ADAPTER_ID: Address =
    address!("FrCvyyGSukMZcLhpU7EneuhfPmqS5p8E2ysnFdwHhopR");
pub const JUPITER_ADAPTER_ID: Address =
    address!("2acqkTDi2VQ4FCZVDB8PeMVLVWnREogE5HA2GxvHdWxu");
pub const MAPLE_ADAPTER_ID: Address =
    address!("Ft2Yvaiqwsjvo1yyYEWvt12YCsDB4kjGBd7vrF8RwwjU");
pub const DRIFT_ADAPTER_ID: Address =
    address!("CVfb8T9tf9WEeus4mKWsxTehVezeY9TGwYsSc3JmxWYz");
pub const REGISTRY_PROGRAM_ID: Address =
    address!("CeyDkRgegNUz2TeFfFjRdL89G9EGGDymiqHoJkeFGcZ4");

pub const REGISTRY_ADAPTER_ENTRY_SEED: &[u8] = b"adapter_entry";
pub const KAMINO_VAULT_STATE_SEED: &[u8] = b"kamino_vault_state";
pub const KAMINO_VAULT_AUTHORITY_SEED: &[u8] = b"kamino_vault_authority";
pub const MARGINFI_VAULT_STATE_SEED: &[u8] = b"marginfi_vault_state";
pub const MARGINFI_VAULT_AUTHORITY_SEED: &[u8] = b"marginfi_vault_authority";
pub const JUPITER_VAULT_STATE_SEED: &[u8] = b"jupiter_vault_state";
pub const JUPITER_VAULT_AUTHORITY_SEED: &[u8] = b"jupiter_vault_authority";
pub const MAPLE_VAULT_STATE_SEED: &[u8] = b"maple_vault_state";
pub const MAPLE_VAULT_AUTHORITY_SEED: &[u8] = b"maple_vault_authority";
pub const DRIFT_VAULT_STATE_SEED: &[u8] = b"drift_vault_state";
pub const DRIFT_VAULT_AUTHORITY_SEED: &[u8] = b"drift_vault_authority";

pub const ADAPTER_ENTRY_APPROVED_STATUS: u8 = 1;
pub const ADAPTER_ENTRY_MIN_LEN: usize = 66;
pub const ADAPTER_ENTRY_STATUS_OFFSET: usize = 33;
pub const ADAPTER_ENTRY_UNDERLYING_MINT_OFFSET: usize = 34;

#[event(discriminator = 0)]
pub struct DepositEvent {
    pub user: Address,
    pub adapter: Address,
    pub amount: u64,
    pub receipt_amount: u64,
    pub timestamp: i64,
}

#[event(discriminator = 1)]
pub struct WithdrawEvent {
    pub user: Address,
    pub adapter: Address,
    pub amount: u64,
    pub receipt_burned: u64,
    pub timestamp: i64,
}

#[event(discriminator = 2)]
pub struct CurrentValueEvent {
    pub user: Address,
    pub adapter: Address,
    pub value: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum YieldAdapterError {
    ZeroDepositAmount = 6000,
    ZeroWithdrawAmount,
    InsufficientReceiptBalance,
    AdapterNotActive,
    MintMismatch,
    AdapterProgramMismatch,
    ArithmeticOverflow,
    ProtocolCpiError,
    Unauthorized,
    PositionNotInitialized,
    InvalidMetadata,
}

pub fn calculate_share_price(total_underlying: u64, total_shares: u64) -> Result<u64, ProgramError> {
    if total_shares == 0 {
        return Ok(1_000_000_000);
    }
    let price = (total_underlying as u128)
        .checked_mul(1_000_000_000)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?
        .checked_div(total_shares as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;
    Ok(price as u64)
}

pub fn underlying_to_shares(underlying_amount: u64, share_price: u64) -> Result<u64, ProgramError> {
    if share_price == 0 {
        return Err(YieldAdapterError::ArithmeticOverflow.into());
    }
    let shares = (underlying_amount as u128)
        .checked_mul(1_000_000_000)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?
        .checked_div(share_price as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;
    Ok(shares as u64)
}

pub fn shares_to_underlying(share_amount: u64, share_price: u64) -> Result<u64, ProgramError> {
    let underlying = (share_amount as u128)
        .checked_mul(share_price as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?
        .checked_div(1_000_000_000)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;
    Ok(underlying as u64)
}

pub fn shares_for_deposit(
    amount: u64,
    total_underlying: u64,
    total_shares: u64,
) -> Result<u64, ProgramError> {
    if total_shares == 0 {
        return Ok(amount);
    }
    let shares = (amount as u128)
        .checked_mul(total_shares as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?
        .checked_div(total_underlying as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;
    Ok(shares as u64)
}

pub fn user_position_underlying_value(
    receipt_token_balance: u64,
    total_underlying: u64,
    total_shares: u64,
) -> Result<u64, ProgramError> {
    if receipt_token_balance == 0 || total_shares == 0 {
        return Ok(0);
    }
    let value = (receipt_token_balance as u128)
        .checked_mul(total_underlying as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?
        .checked_div(total_shares as u128)
        .ok_or(YieldAdapterError::ArithmeticOverflow)?;
    Ok(value as u64)
}

pub fn verify_protocol_program_account(
    program: &RemainingAccount,
    expected: Address,
) -> Result<(), ProgramError> {
    require_keys_eq!(*program.address(), expected, YieldAdapterError::AdapterProgramMismatch);
    require!(program.executable(), YieldAdapterError::ProtocolCpiError);
    Ok(())
}

pub fn record_protocol_routing(
    routed_total: &mut PodU64,
    amount: u64,
    remaining: RemainingAccounts<'_>,
    expected: Address,
) -> Result<(), ProgramError> {
    let mut iter = remaining.iter();
    if let Some(item) = iter.next() {
        let prog = item?;
        verify_protocol_program_account(&prog, expected)?;
        let current: u64 = (*routed_total).into();
        *routed_total = current
            .checked_add(amount)
            .ok_or(YieldAdapterError::ArithmeticOverflow)?
            .into();
    }
    Ok(())
}

/// Minimum byte length for a reference adapter vault state account (1-byte disc + head).
pub const REFERENCE_VAULT_MIN_LEN: usize = 81;
/// Minimum byte length for a reference adapter position account (1-byte disc + head).
pub const ADAPTER_POSITION_MIN_LEN: usize = 89;
const VAULT_TOTAL_UNDERLYING_OFFSET: usize = 65;
const VAULT_TOTAL_SHARES_OFFSET: usize = 73;
const POSITION_RECEIPT_BALANCE_OFFSET: usize = 81;

fn read_u64_le(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    let end = offset
        .checked_add(8)
        .ok_or(YieldAdapterError::InvalidMetadata)?;
    require!(data.len() >= end, YieldAdapterError::InvalidMetadata);
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[offset..end]);
    Ok(u64::from_le_bytes(bytes))
}

/// Read `total_underlying` and `total_shares` from a reference adapter vault account.
pub fn read_reference_vault_totals(data: &[u8]) -> Result<(u64, u64), ProgramError> {
    require!(
        data.len() >= REFERENCE_VAULT_MIN_LEN,
        YieldAdapterError::InvalidMetadata
    );
    Ok((
        read_u64_le(data, VAULT_TOTAL_UNDERLYING_OFFSET)?,
        read_u64_le(data, VAULT_TOTAL_SHARES_OFFSET)?,
    ))
}

/// Read `receipt_token_balance` from a reference adapter position account.
pub fn read_adapter_position_receipt(data: &[u8]) -> Result<u64, ProgramError> {
    require!(
        data.len() >= ADAPTER_POSITION_MIN_LEN,
        YieldAdapterError::PositionNotInitialized
    );
    read_u64_le(data, POSITION_RECEIPT_BALANCE_OFFSET)
}

pub fn read_adapter_entry_underlying_mint(data: &[u8]) -> Result<Address, ProgramError> {
    require!(
        data.len() >= ADAPTER_ENTRY_MIN_LEN,
        YieldAdapterError::InvalidMetadata
    );
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[ADAPTER_ENTRY_UNDERLYING_MINT_OFFSET..ADAPTER_ENTRY_UNDERLYING_MINT_OFFSET + 32]);
    Ok(Address::from(bytes))
}

pub fn adapter_entry_is_approved(data: &[u8]) -> Result<(), ProgramError> {
    require!(
        data.len() >= ADAPTER_ENTRY_MIN_LEN,
        YieldAdapterError::InvalidMetadata
    );
    require!(
        data[ADAPTER_ENTRY_STATUS_OFFSET] == ADAPTER_ENTRY_APPROVED_STATUS,
        YieldAdapterError::InvalidMetadata
    );
    Ok(())
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    // -------------------------------------------------------------------
    // calculate_share_price
    // -------------------------------------------------------------------

    #[test]
    fn test_share_price_initial() {
        std::assert_eq!(calculate_share_price(0, 0).unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_share_price_after_deposit() {
        std::assert_eq!(calculate_share_price(1000, 1000).unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_share_price_with_yield() {
        std::assert_eq!(calculate_share_price(1000, 500).unwrap(), 2_000_000_000);
    }

    #[test]
    fn test_share_price_with_loss() {
        std::assert_eq!(calculate_share_price(500, 1000).unwrap(), 500_000_000);
    }

    #[test]
    fn test_share_price_large_values_no_panic() {
        let price = calculate_share_price(1_000_000_000_000_000u64, 3_000_000u64).unwrap();
        std::assert!(price > 0);
    }

    // -------------------------------------------------------------------
    // underlying_to_shares
    // -------------------------------------------------------------------

    #[test]
    fn test_underlying_to_shares_at_par() {
        std::assert_eq!(underlying_to_shares(1000, 1_000_000_000).unwrap(), 1000);
    }

    #[test]
    fn test_underlying_to_shares_half_price() {
        std::assert_eq!(underlying_to_shares(1000, 500_000_000).unwrap(), 2000);
    }

    #[test]
    fn test_underlying_to_shares_zero_price() {
        std::assert!(underlying_to_shares(1000, 0).is_err());
    }

    // -------------------------------------------------------------------
    // shares_to_underlying
    // -------------------------------------------------------------------

    #[test]
    fn test_shares_to_underlying_at_par() {
        std::assert_eq!(shares_to_underlying(1000, 1_000_000_000).unwrap(), 1000);
    }

    #[test]
    fn test_shares_to_underlying_double_price() {
        std::assert_eq!(shares_to_underlying(1000, 2_000_000_000).unwrap(), 2000);
    }

    // -------------------------------------------------------------------
    // shares_for_deposit
    // -------------------------------------------------------------------

    #[test]
    fn test_first_deposit_onetoone() {
        std::assert_eq!(shares_for_deposit(1000, 0, 0).unwrap(), 1000);
    }

    #[test]
    fn test_second_deposit_proportional() {
        std::assert_eq!(shares_for_deposit(500, 1000, 1000).unwrap(), 500);
    }

    #[test]
    fn test_deposit_when_pool_has_yield() {
        std::assert_eq!(shares_for_deposit(1000, 2000, 1000).unwrap(), 500);
    }

    #[test]
    fn test_deposit_rounds_down() {
        std::assert_eq!(shares_for_deposit(1000, 1000, 3).unwrap(), 3);
    }

    // -------------------------------------------------------------------
    // user_position_underlying_value
    // -------------------------------------------------------------------

    #[test]
    fn test_zero_balance() {
        std::assert_eq!(user_position_underlying_value(0, 1000, 1000).unwrap(), 0);
    }

    #[test]
    fn test_zero_shares() {
        std::assert_eq!(user_position_underlying_value(500, 0, 0).unwrap(), 0);
    }

    #[test]
    fn test_position_value_par() {
        std::assert_eq!(user_position_underlying_value(500, 1000, 1000).unwrap(), 500);
    }

    #[test]
    fn test_position_value_after_loss() {
        std::assert_eq!(user_position_underlying_value(500, 500, 1000).unwrap(), 250);
    }

    // -------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------

    #[test]
    fn test_large_numbers_no_overflow() {
        let total = 1_000_000_000_000u64;
        let shares = 3_000_000_000u64;
        let price = calculate_share_price(total, shares).unwrap();
        std::assert!(price > 0);

        let deposit = 500_000_000_000u64;
        let result = shares_for_deposit(deposit, total, shares).unwrap();
        std::assert!(result > 0);

        let value = user_position_underlying_value(shares / 2, total, shares).unwrap();
        std::assert!(value > 0);
    }
}
