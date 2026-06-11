use {
    crate::{
        error::DriftAdapterError, protocol,
        state::{AdapterPosition, DriftVaultState, VaultAuthorityPda},
        UNSTAKE_COOLDOWN_SECONDS,
    },
    quasar_lang::{cpi::Seed, prelude::*, sysvars::Sysvar},
    quasar_spl::prelude::*,
    yield_adapter_trait::{user_position_underlying_value, WithdrawEvent, YieldAdapterError},
};

#[derive(Accounts)]
pub struct Withdraw {
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
        address = AdapterPosition::seeds(user.address()),
        constraints(user_position.owner == *user.address()) @ YieldAdapterError::Unauthorized,
    )]
    pub user_position: Account<AdapterPosition>,
    #[account(
        mut,
        constraints(*user_token_account.owner() == *user.address()) @ YieldAdapterError::Unauthorized,
        constraints(*user_token_account.mint() == vault_state.underlying_mint) @ YieldAdapterError::MintMismatch,
    )]
    pub user_token_account: Account<Token>,
    #[account(
        mut,
        constraints(*vault_token_account.mint() == vault_state.underlying_mint) @ YieldAdapterError::MintMismatch,
        constraints(*vault_token_account.owner() == *vault_authority.address()) @ YieldAdapterError::Unauthorized,
    )]
    pub vault_token_account: Account<Token>,
    #[account(address = VaultAuthorityPda::seeds())]
    pub vault_authority: UncheckedAccount,
    pub token_program: Program<TokenProgram>,
}

impl Withdraw {
    pub fn handler(
        &mut self,
        shares_to_burn: u64,
        bumps: &WithdrawBumps,
        remaining: RemainingAccounts<'_>,
    ) -> Result<(), ProgramError> {
        require!(shares_to_burn > 0, YieldAdapterError::ZeroWithdrawAmount);
        require!(
            self.user_position.receipt_token_balance >= shares_to_burn,
            YieldAdapterError::InsufficientReceiptBalance
        );

        let clock = Clock::get()?;
        let now = clock.unix_timestamp.get();

        let last_request: i64 = self.user_position.last_withdraw_request.into();
        if last_request > 0 {
            let elapsed = now.saturating_sub(last_request);
            require!(
                elapsed >= UNSTAKE_COOLDOWN_SECONDS,
                DriftAdapterError::CooldownNotElapsed
            );
        }

        let underlying_amount = user_position_underlying_value(
            shares_to_burn,
            self.vault_state.total_underlying.into(),
            self.vault_state.total_shares.into(),
        )?;

        protocol::on_withdraw(
            &mut self.vault_state,
            underlying_amount,
            self.vault_authority.to_account_view(),
            bumps.vault_authority,
            self.vault_token_account.to_account_view(),
            self.token_program.to_account_view(),
            remaining,
        )?;

        let bump = [bumps.vault_authority];
        let seeds = [
            Seed::from(b"drift_vault_authority" as &[u8]),
            Seed::from(bump.as_ref()),
        ];

        self.token_program
            .transfer(
                &self.vault_token_account,
                &self.user_token_account,
                &self.vault_authority,
                underlying_amount,
            )
            .invoke_signed(&seeds)?;

        {
            let total_underlying: u64 = self.vault_state.total_underlying.into();
            self.vault_state.total_underlying = total_underlying
                .checked_sub(underlying_amount)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }
        {
            let total_shares: u64 = self.vault_state.total_shares.into();
            self.vault_state.total_shares = total_shares
                .checked_sub(shares_to_burn)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }

        {
            let receipt_balance: u64 = self.user_position.receipt_token_balance.into();
            self.user_position.receipt_token_balance = receipt_balance
                .checked_sub(shares_to_burn)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }
        {
            let withdrawn: u64 = self.user_position.withdrawn_amount.into();
            self.user_position.withdrawn_amount = withdrawn
                .checked_add(underlying_amount)
                .ok_or(YieldAdapterError::ArithmeticOverflow)?
                .into();
        }
        self.user_position.last_updated = now.into();
        self.user_position.last_withdraw_request = now.into();

        emit!(WithdrawEvent {
            user: *self.user.address(),
            adapter: crate::ID,
            amount: underlying_amount,
            receipt_burned: shares_to_burn,
            timestamp: now,
        });

        Ok(())
    }
}
