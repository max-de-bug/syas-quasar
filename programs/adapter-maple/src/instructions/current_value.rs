use {
    crate::state::{AdapterPosition, MapleVaultState},
    quasar_lang::{prelude::*, sysvars::Sysvar},
    yield_adapter_trait::{user_position_underlying_value, CurrentValueEvent, YieldAdapterError},
};

#[derive(Accounts)]
pub struct CurrentValue {
    pub user: Signer,
    #[account(mut, address = MapleVaultState::seeds())]
    pub vault_state: Account<MapleVaultState>,
    #[account(
        address = AdapterPosition::seeds(user.address()),
        constraints(user_position.owner == *user.address()) @ YieldAdapterError::Unauthorized,
    )]
    pub user_position: Account<AdapterPosition>,
}

impl CurrentValue {
    pub fn handler(&mut self, _remaining: RemainingAccounts<'_>) -> Result<(), ProgramError> {
        let value = user_position_underlying_value(
            self.user_position.receipt_token_balance.into(),
            self.vault_state.total_underlying.into(),
            self.vault_state.total_shares.into(),
        )?;

        let clock = Clock::get()?;
        let now = clock.unix_timestamp.get();

        emit!(CurrentValueEvent {
            user: *self.user.address(),
            adapter: crate::ID,
            value,
            timestamp: now,
        });

        Ok(())
    }
}
