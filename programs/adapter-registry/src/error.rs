use quasar_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    Unauthorized = 6200,
    NameTooLong,
    UriTooLong,
    InvalidStatus,
    AlreadyRegistered,
    NoPendingTransfer,
    NotPendingAuthority,
}
