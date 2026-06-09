use {
    crate::state::{AdapterPosition, JupiterVaultState, VaultAuthorityPda},
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
        address = JupiterVaultState::seeds(),
        constraints(vault_state.is_active.into()) @ YieldAdapterError::AdapterNotActive,
    )]
    pub vault_state: Account<JupiterVaultState>,
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
    ) -> Result<(), ProgramError> {
        require!(shares_to_burn > 0, YieldAdapterError::ZeroWithdrawAmount);
        require!(
            self.user_position.receipt_token_balance >= shares_to_burn,
            YieldAdapterError::InsufficientReceiptBalance
        );

        let underlying_amount = user_position_underlying_value(
            shares_to_burn,
            self.vault_state.total_underlying.into(),
            self.vault_state.total_shares.into(),
        )?;

        let bump = [bumps.vault_authority];
        let seeds = [
            Seed::from(b"jupiter_vault_authority" as &[u8]),
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

        let clock = Clock::get()?;
        let now = clock.unix_timestamp.get();

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
