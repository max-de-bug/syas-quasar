use {
    crate::{
        protocol::KAMINO_LEND_ID,
        state::{KaminoVaultState, KaminoVaultStateInner},
    },
    quasar_lang::prelude::*,
};

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        mut,
        init,
        payer = authority,
        address = KaminoVaultState::seeds()
    )]
    pub vault_state: Account<KaminoVaultState>,
    pub rent: Sysvar<Rent>,
    pub system_program: Program<SystemProgram>,
}

impl Initialize {
    pub fn handler(
        &mut self,
        underlying_mint: Address,
        bumps: &InitializeBumps,
    ) -> Result<(), ProgramError> {
        self.vault_state.set_inner(KaminoVaultStateInner {
            authority: *self.authority.address(),
            underlying_mint,
            total_underlying: 0,
            total_shares: 0,
            protocol_program_id: KAMINO_LEND_ID,
            protocol_routed_underlying: 0,
            last_yield_sync_ts: 0,
            is_active: true,
            bump: bumps.vault_state,
        });
        Ok(())
    }
}
