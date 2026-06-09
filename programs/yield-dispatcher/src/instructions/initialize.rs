use {
    crate::{
        events::DispatcherInitializedEvent,
        state::{DispatcherState, DispatcherStateInner},
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
        address = DispatcherState::seeds()
    )]
    pub dispatcher_state: Account<DispatcherState>,
    pub registry_program: UncheckedAccount,
    pub rent: Sysvar<Rent>,
    pub system_program: Program<SystemProgram>,
}

impl Initialize {
    pub fn handler(&mut self, bumps: &InitializeBumps) -> Result<(), ProgramError> {
        self.dispatcher_state.set_inner(DispatcherStateInner {
            authority: *self.authority.address(),
            registry_program_id: *self.registry_program.address(),
            total_deposits: 0,
            total_withdrawals: 0,
            is_paused: false,
            bump: bumps.dispatcher_state,
        });

        let clock = <Clock as quasar_lang::sysvars::Sysvar>::get()?;
        let now = clock.unix_timestamp.get();

        emit!(DispatcherInitializedEvent {
            authority: *self.authority.address(),
            registry_program_id: *self.registry_program.address(),
            timestamp: now,
        });

        Ok(())
    }
}
