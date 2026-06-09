use {
    quasar_lang::prelude::*,
    yield_adapter_trait::{MAX_ADAPTER_NAME_LEN, MAX_METADATA_URI_LEN},
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdapterStatus {
    Proposed = 0,
    Approved = 1,
    Revoked = 2,
}

impl AdapterStatus {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

#[account(discriminator = 1, set_inner)]
#[seeds(b"registry_state")]
pub struct RegistryState {
    pub authority: Address,
    pub pending_authority: Option<Address>,
    pub total_proposed: u64,
    pub total_approved: u64,
    pub bump: u8,
}

#[account(discriminator = 2, set_inner)]
#[seeds(b"adapter_entry", adapter_program: Address)]
pub struct AdapterEntry {
    pub adapter_program_id: Address,
    pub status: u8,
    pub underlying_mint: Address,
    pub proposer: Address,
    pub proposed_at: i64,
    pub approved_at: i64,
    pub revoked_at: i64,
    pub bump: u8,
    pub name_len: u8,
    pub name_buf: [u8; MAX_ADAPTER_NAME_LEN],
    pub metadata_uri_len: u8,
    pub metadata_uri_buf: [u8; MAX_METADATA_URI_LEN],
}
