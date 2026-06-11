import { Keypair, SystemProgram } from "@solana/web3.js";
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
  buildAdapterCurrentValue,
  RENT_SYSVAR,
} from "../helpers/quasar-client";
import { TestProvider } from "../helpers/provider";
import { isMainnetFork, KAMINO_PROGRAM_ID, PROGRAM_IDS, MAINNET_USDC_MINT, TOKEN_PROGRAM_ID } from "../helpers/constants";
import { adapterUserPositionPda, findPda, getTokenBalance } from "../helpers";
import {
  KAMINO_FORK_ACCOUNTS,
  buildKaminoDepositRemainingAccounts,
  buildKaminoWithdrawRemainingAccounts,
} from "../helpers/kamino";
import { expect } from "chai";

describe("adapter-kamino", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  if (isMainnetFork()) {
    it("loads Kamino K-Lend program from mainnet fork", async () => {
      await assertProtocolProgramLoaded(
        provider.connection,
        KAMINO_PROGRAM_ID,
        "Kamino K-Lend"
      );
    });
  }

  it("deposit → current_value → withdraw", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterKamino,
      vaultStateSeed: "kamino_vault_state",
      vaultAuthoritySeed: "kamino_vault_authority",
    });
  });

  if (isMainnetFork()) {
    it("CPI deposit → withdraw with Kamino remaining accounts", async () => {
      const adapterProgramId = PROGRAM_IDS.adapterKamino;
      const depositAmount = 500_000;
      const withdrawShares = 250_000;

      // Setup vault PDAs
      const [vaultStatePda] = findPda(
        [Buffer.from("kamino_vault_state")],
        adapterProgramId
      );
      const [vaultAuthorityPda] = findPda(
        [Buffer.from("kamino_vault_authority")],
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

      // Create vault's collateral (kUSDC) token account
      const vaultCollateralAccount = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        KAMINO_FORK_ACCOUNTS.collateralMint,
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

      // Build deposit instruction with Kamino CPI remaining accounts
      const depositRemainingAccounts = buildKaminoDepositRemainingAccounts(
        vaultCollateralAccount.address
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

      // Build withdraw instruction with Kamino CPI remaining accounts
      const withdrawRemainingAccounts = buildKaminoWithdrawRemainingAccounts(
        vaultCollateralAccount.address
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

      // Verify user received tokens back (approximately)
      // Some fees/slippage may apply due to Kamino's actual protocol interaction
      const userBalanceAfter = await getTokenBalance(
        provider.connection,
        userTokenAccount
      );
      expect(userBalanceAfter).to.be.greaterThan(depositAmount);
    });
  }
});
