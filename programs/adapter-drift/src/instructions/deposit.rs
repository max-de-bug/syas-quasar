use {
    crate::{
        protocol,
        state::{AdapterPosition, AdapterPositionInner, DriftVaultState, VaultAuthorityPda},
    },
    quasar_lang::{prelude::*, sysvars::Sysvar},
    quasar_spl::prelude::*,
    yield_adapter_trait::{shares_for_deposit, DepositEvent, YieldAdapterError},
};

#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub user: Signer,
    #[account(
        mut,
        address = DriftVaultState::seeds(),
        constraints(vault_state.is_active.into()) @ YieldAdapterError::AdapterNotActive,
    )]
    pub vault_state: Account<DriftVaultState>,
    #[account(
        mut,
        init(idempotent),
        payer = user,
        address = AdapterPosition::seeds(user.address())
    )]
    pub user_position: Account<AdapterPosition>,
    #[account(
        mut,
        constraints(*user_token_account.owner() == *user.address()) @ YieldAdapterError::Unauthorized,
        constraints(*user_token_account.mint() == vault_state.underlying_mint) @ YieldAdapterError::MintMismatch,
    )]
    pub user_token_account: Account<Token>,
    #[account(address = VaultAuthorityPda::seeds())]
    pub vault_authority: UncheckedAccount,
    #[account(
        mut,
        constraints(*vault_token_account.mint() == vault_state.underlying_mint) @ YieldAdapterError::MintMismatch,
        constraints(*vault_token_account.owner() == *vault_authority.address()) @ YieldAdapterError::Unauthorized,
    )]
    pub vault_token_account: Account<Token>,
    pub token_program: Program<TokenProgram>,
    pub system_program: Program<SystemProgram>,
}

impl Deposit {
    pub fn handler(
        &mut self,
        amount: u64,
        bumps: &DepositBumps,
        remaining: RemainingAccounts<'_>,
    ) -> Result<(), ProgramError> {
        require!(amount > 0, YieldAdapterError::ZeroDepositAmount);

        let shares = shares_for_deposit(
            amount,
            self.vault_state.total_underlying.into(),
            self.vault_state.total_shares.into(),
        )?;

        self.token_program
            .transfer(
                &self.user_token_account,
                &self.vault_token_account,
                &self.user,
                amount,
            )
            .invoke()?;

        {
            let total_underlying: u64 = self.vault_state.total_underlying.into();
            self.vault_state.total_underlying = total_underlying
                .checked_add(amount)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }
        {
            let total_shares: u64 = self.vault_state.total_shares.into();
            self.vault_state.total_shares = total_shares
                .checked_add(shares)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }

        protocol::on_deposit(&mut self.vault_state, amount, remaining)?;

        let clock = Clock::get()?;
        let now = clock.unix_timestamp.get();

        if self.user_position.owner == Address::default() {
            self.user_position.set_inner(AdapterPositionInner {
                owner: *self.user.address(),
                adapter_program_id: crate::ID,
                deposited_amount: 0,
                withdrawn_amount: 0,
                receipt_token_balance: 0,
                last_updated: 0,
                last_withdraw_request: 0,
                bump: bumps.user_position,
            });
        }

        {
            let deposited: u64 = self.user_position.deposited_amount.into();
            self.user_position.deposited_amount = deposited
                .checked_add(amount)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }
        {
            let receipt_balance: u64 = self.user_position.receipt_token_balance.into();
            self.user_position.receipt_token_balance = receipt_balance
                .checked_add(shares)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }
        self.user_position.last_updated = now.into();

        emit!(DepositEvent {
            user: *self.user.address(),
            adapter: crate::ID,
            amount,
            receipt_amount: shares,
            timestamp: now,
        });

        let _ = bumps;
        Ok(())
    }
}
