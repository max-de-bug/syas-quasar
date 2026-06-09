use quasar_lang::prelude::*;

#[account(discriminator = 1, set_inner)]
#[seeds(b"jupiter_vault_state")]
pub struct JupiterVaultState {
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

#[derive(Seeds)]
#[seeds(b"jupiter_vault_authority")]
pub struct VaultAuthorityPda;
