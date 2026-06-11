import { Keypair, SystemProgram, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { expect } from "chai";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";

import {
  assertProtocolProgramLoaded,
  runAdapterDepositWithdrawFlow,
  fundUserUsdcOnFork,
  createVaultTokenAccount,
  initializeAdapterVault,
} from "../helpers/adapter";
import {
  buildAdapterDeposit,
  buildAdapterWithdraw,
  RENT_SYSVAR,
} from "../helpers/quasar-client";
import { TestProvider } from "../helpers/provider";
import { isMainnetFork, JUPITER_PERPS_PROGRAM_ID, PROGRAM_IDS, MAINNET_USDC_MINT, TOKEN_PROGRAM_ID } from "../helpers/constants";
import { adapterUserPositionPda, findPda, getTokenBalance } from "../helpers";
import {
  JUPITER_FORK_ACCOUNTS,
  buildJupiterDepositRemainingAccounts,
  buildJupiterWithdrawRemainingAccounts,
} from "../helpers/jupiter";

describe("adapter-jupiter", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  if (isMainnetFork()) {
    it("loads Jupiter Perpetuals program from mainnet fork", async () => {
      await assertProtocolProgramLoaded(
        provider.connection,
        JUPITER_PERPS_PROGRAM_ID,
        "Jupiter Perpetuals"
      );
    });
  }

  it("deposit → current_value → withdraw", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterJupiter,
      vaultStateSeed: "jupiter_vault_state",
      vaultAuthoritySeed: "jupiter_vault_authority",
    });
  });

  if (isMainnetFork()) {
    it.skip("CPI deposit → withdraw with Jupiter remaining accounts", async () => {
      /**
       * SKIPPED: Jupiter account addresses need verification
       * 
       * To enable this test, update JUPITER_FORK_ACCOUNTS in tests/helpers/jupiter.ts
       * with the correct on-chain addresses. Required accounts:
       * - perpetuals: Perpetuals state account (lookup from Jupiter program)
       * - pool: USDC pool state (lookup from perpetuals)
       * - dovesPrice: Doves oracle price feed
       * - pythPrice: Pyth USDC/USD price feed
       * - lpTokenMint: LP token mint for the pool
       * - eventAuthority: Event authority for Jupiter
       * - transferAuthority: Transfer authority for liquidity ops
       * 
       * Also update run-mainnet-fork-tests.sh to clone these accounts.
       * 
       * See Jupiter docs: https://github.com/jupiter-aggregator/perpetuals
       */

      const adapterProgramId = PROGRAM_IDS.adapterJupiter;
      const depositAmount = 500_000;
      const withdrawShares = 250_000;

      // Setup vault PDAs
      const [vaultStatePda] = findPda(
        [Buffer.from("jupiter_vault_state")],
        adapterProgramId
      );
      const [vaultAuthorityPda] = findPda(
        [Buffer.from("jupiter_vault_authority")],
        adapterProgramId
      );

      // Initialize adapter vault
      await initializeAdapterVault(
        provider,
        adapterProgramId,
        vaultStatePda,
        MAINNET_USDC_MINT
      );

      // Create vault's USDC token account
      const vaultTokenAccount = await createVaultTokenAccount(
        provider,
        payer,
        MAINNET_USDC_MINT,
        vaultAuthorityPda
      );

      // Create vault's LP token account
      const vaultLpAccount = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        JUPITER_FORK_ACCOUNTS.lpTokenMint,
        vaultAuthorityPda,
        true
      );

      // Fund user with USDC
      const userTokenAccount = await fundUserUsdcOnFork(
        provider,
        payer,
        provider.publicKey,
        depositAmount * 2
      );

      // User position PDA
      const userPositionPda = adapterUserPositionPda(
        provider.publicKey,
        adapterProgramId
      );

      // Build deposit instruction with Jupiter CPI remaining accounts
      const transferAuthority = vaultAuthorityPda; // Use vault authority as transfer authority
      const depositRemainingAccounts = buildJupiterDepositRemainingAccounts(
        transferAuthority,
        vaultLpAccount.address
      );

      const depositIx = buildAdapterDeposit(
        adapterProgramId,
        {
          user: provider.publicKey,
          vaultState: vaultStatePda,
          userPosition: userPositionPda,
          userTokenAccount,
          vaultAuthority: vaultAuthorityPda,
          vaultTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        },
        depositAmount,
        depositRemainingAccounts
      );

      // Execute deposit
      await provider.sendIx(depositIx);

      // Verify vault received tokens
      const vaultBalance = await getTokenBalance(
        provider.connection,
        vaultTokenAccount
      );
      expect(vaultBalance).to.be.at.least(depositAmount);

      // Build withdraw instruction with Jupiter CPI remaining accounts
      const withdrawRemainingAccounts = buildJupiterWithdrawRemainingAccounts(
        transferAuthority,
        vaultLpAccount.address
      );

      const withdrawIx = buildAdapterWithdraw(
        adapterProgramId,
        {
          user: provider.publicKey,
          vaultState: vaultStatePda,
          userPosition: userPositionPda,
          userTokenAccount,
          vaultTokenAccount,
          vaultAuthority: vaultAuthorityPda,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
        withdrawShares,
        withdrawRemainingAccounts
      );

      // Execute withdraw
      await provider.sendIx(withdrawIx);

      // Verify user received tokens back
      const userBalanceAfter = await getTokenBalance(
        provider.connection,
        userTokenAccount
      );
      expect(userBalanceAfter).to.be.greaterThan(depositAmount);
    });
  }
});
