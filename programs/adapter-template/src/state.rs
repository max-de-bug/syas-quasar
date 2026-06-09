use quasar_lang::prelude::*;

/// Per-vault state account.
///
/// ### Customizing for your protocol
/// Add protocol-specific fields here, for example:
/// - `pool_id: Address` — which pool/lending-market this vault targets
/// - `reserve_index: u16` — which reserve in a multi-reserve protocol
///
/// Fields you may remove:
/// - `protocol_program_id` — if no external CPI
/// - `last_yield_sync_ts` — if no periodic yield sync needed
#[account(discriminator = 1, set_inner)]
#[seeds(b"template_vault_state")]
pub struct TemplateVaultState {
    pub authority: Address,
    pub underlying_mint: Address,
    pub total_underlying: u64,
    pub total_shares: u64,
    pub protocol_program_id: Address,
    pub protocol_routed_underlying: u64,
    pub last_yield_sync_ts: i64,
    pub is_active: bool,
    pub bump: u8,
}

/// Per-user position tracking account.
#[account(discriminator = 2, set_inner)]
#[seeds(b"adapter_position", user: Address)]
pub struct AdapterPosition {
    pub owner: Address,
    pub adapter_program_id: Address,
    pub deposited_amount: u64,
    pub withdrawn_amount: u64,
    pub receipt_token_balance: u64,
    pub last_updated: i64,
    pub last_withdraw_request: i64,
    pub bump: u8,
}

/// Vault authority PDA — signs for protocol CPI and vault token transfers.
/// The seed is derived from `template_vault_authority`.
#[derive(Seeds)]
#[seeds(b"template_vault_authority")]
pub struct VaultAuthorityPda;
