use quasar_lang::prelude::*;

#[event(discriminator = 0)]
pub struct DispatchDepositEvent {
    pub user: Address,
    pub adapter_program_id: Address,
    pub amount: u64,
    pub timestamp: i64,
}

#[event(discriminator = 1)]
pub struct DispatchWithdrawEvent {
    pub user: Address,
    pub adapter_program_id: Address,
    pub amount: u64,
    pub timestamp: i64,
}

#[event(discriminator = 2)]
pub struct DispatchCurrentValueEvent {
    pub user: Address,
    pub adapter_program_id: Address,
    pub value: u64,
    pub timestamp: i64,
}

#[event(discriminator = 3)]
pub struct DispatcherInitializedEvent {
    pub authority: Address,
    pub registry_program_id: Address,
    pub timestamp: i64,
}
