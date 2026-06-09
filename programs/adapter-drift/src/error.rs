use quasar_lang::prelude::*;

#[error_code]
pub enum DriftAdapterError {
    CooldownNotElapsed = 7000,
}
