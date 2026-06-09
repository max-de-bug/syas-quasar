use quasar_lang::prelude::*;

#[account(discriminator = 1, set_inner)]
#[seeds(b"dispatcher_state")]
pub struct DispatcherState {
    pub authority: Address,
    pub registry_program_id: Address,
    pub total_deposits: u64,
    pub total_withdrawals: u64,
    pub is_paused: bool,
    pub bump: u8,
}

#[account(discriminator = 2, set_inner)]
#[seeds(b"user_position", user: Address, adapter_program: Address)]
pub struct UserPosition {
    pub owner: Address,
    pub adapter_program_id: Address,
    pub deposited_amount: u64,
    pub withdrawn_amount: u64,
    pub receipt_token_balance: u64,
    pub last_updated: i64,
    pub bump: u8,
}
