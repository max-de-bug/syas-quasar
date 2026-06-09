import { Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { expect } from "chai";

import {
  initializeAdapterVault,
  createVaultTokenAccount,
  runAdapterDepositWithdrawFlow,
} from "../helpers/adapter";
import { TestProvider } from "../helpers/provider";
import { PROGRAM_IDS, TOKEN_PROGRAM_ID, isMainnetFork, SYRUP_USDC_MINT } from "../helpers/constants";
import { createTestMint, createTestTokenAccount, findPda, adapterUserPositionPda } from "../helpers";
import { buildAdapterDeposit } from "../helpers/quasar-client";

describe("adapter-maple", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  const vaultStateSeed = "maple_vault_state";
  const vaultAuthoritySeed = "maple_vault_authority";

  let vaultStatePda: import("@solana/web3.js").PublicKey;
  let vaultAuthorityPda: import("@solana/web3.js").PublicKey;
  let underlyingMint: import("@solana/web3.js").PublicKey;
  let vaultTokenAccount: import("@solana/web3.js").PublicKey;

  before(async () => {
    [vaultStatePda] = findPda(
      [Buffer.from(vaultStateSeed)],
      PROGRAM_IDS.adapterMaple
    );
    [vaultAuthorityPda] = findPda(
      [Buffer.from(vaultAuthoritySeed)],
      PROGRAM_IDS.adapterMaple
    );

    underlyingMint = isMainnetFork()
      ? SYRUP_USDC_MINT
      : await createTestMint(provider.connection, payer, 6);

    await initializeAdapterVault(
      provider,
      PROGRAM_IDS.adapterMaple,
      vaultStatePda,
      underlyingMint,
    );
    vaultTokenAccount = await createVaultTokenAccount(
      provider,
      payer,
      underlyingMint,
      vaultAuthorityPda,
    );
  });

  it("deposit → current_value → withdraw (syrupUSDC model)", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterMaple,
      vaultStateSeed,
      vaultAuthoritySeed,
      ...(isMainnetFork() ? { underlyingMint: SYRUP_USDC_MINT } : {}),
    });
  });

  it("rejects zero amount deposit", async () => {
    const userTokenAccount = isMainnetFork()
      ? await getOrCreateAssociatedTokenAccount(
          provider.connection, payer, underlyingMint, provider.publicKey
        ).then(a => a.address)
      : await createTestTokenAccount(
          provider.connection, underlyingMint, provider.publicKey, payer
        );

    const userPositionPda = adapterUserPositionPda(
      provider.publicKey,
      PROGRAM_IDS.adapterMaple,
    );

    const depositIx = buildAdapterDeposit(
      PROGRAM_IDS.adapterMaple,
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
      0,
    );

    try {
      await provider.sendIx(depositIx);
      expect.fail("Should have rejected zero deposit");
    } catch (err: unknown) {
      expect(String(err)).to.contain("custom program error");
    }
  });
});
