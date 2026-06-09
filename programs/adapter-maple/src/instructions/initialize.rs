use {
    crate::state::{MapleVaultState, MapleVaultStateInner},
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
        address = MapleVaultState::seeds()
    )]
    pub vault_state: Account<MapleVaultState>,
    pub rent: Sysvar<Rent>,
    pub system_program: Program<SystemProgram>,
}

impl Initialize {
    pub fn handler(
        &mut self,
        underlying_mint: Address,
        bumps: &InitializeBumps,
    ) -> Result<(), ProgramError> {
        self.vault_state.set_inner(MapleVaultStateInner {
            authority: *self.authority.address(),
            underlying_mint,
            total_underlying: 0,
            total_shares: 0,
            protocol_routed_underlying: 0,
            is_active: true,
            bump: bumps.vault_state,
        });
        Ok(())
    }
}
