use quasar_lang::prelude::*;

#[error_code]
pub enum DispatcherError {
    DispatcherPaused = 6100,
    AdapterNotApproved,
    ZeroAmount,
    Unauthorized,
    AdapterCpiError,
    RegistryMismatch,
}
