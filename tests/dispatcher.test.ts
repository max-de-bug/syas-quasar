import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import {
  createTestTokenAccount,
  mintTestTokens,
  getTokenBalance,
  findPda,
} from "./helpers";
import {
  fundUserUsdcOnFork,
  resolveUnderlyingMint,
} from "./helpers/adapter";
import { TestProvider } from "./helpers/provider";
import { isMainnetFork, MAINNET_USDC_MINT, PROGRAM_IDS } from "./helpers/constants";
import {
  RENT_SYSVAR,
  buildDispatcherInitialize,
  buildDispatcherDeposit,
  buildDispatcherWithdraw,
  buildProposeAdapter,
  deserializeDispatcherState,
  deserializeUserPosition,
} from "./helpers/quasar-client";
import {
  ensureRegistryInitialized,
  setupApprovedKaminoForDispatcher,
  userPositionPda,
} from "./helpers/dispatcher";

describe("yield-dispatcher", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet;
  const authority = provider.wallet;

  let dispatcherStatePda: PublicKey;
  let usdcMint: PublicKey;

  async function fundUserForTest(
    mint: PublicKey,
    amount: number
  ): Promise<PublicKey> {
    if (isMainnetFork() && mint.equals(MAINNET_USDC_MINT)) {
      return fundUserUsdcOnFork(
        provider,
        payer,
        authority.publicKey,
        amount * 2
      );
    }
    const ata = await createTestTokenAccount(
      provider.connection,
      mint,
      authority.publicKey,
      payer
    );
    await mintTestTokens(provider.connection, mint, ata, payer, amount * 2);
    return ata;
  }

  before(async () => {
    [dispatcherStatePda] = findPda(
      [Buffer.from("dispatcher_state")],
      PROGRAM_IDS.yieldDispatcher
    );

    usdcMint = await resolveUnderlyingMint(provider, payer);
    await ensureRegistryInitialized(provider);
  });

  it("initializes the dispatcher", async () => {
    try {
      const ix = buildDispatcherInitialize(
        authority.publicKey,
        dispatcherStatePda,
        PROGRAM_IDS.adapterRegistry,
        RENT_SYSVAR,
        SystemProgram.programId,
      );
      await provider.sendIx(ix);
    } catch (e: unknown) {
      const msg = String(e);
      if (!msg.includes("already in use") && !msg.includes("0x0")) {
        throw e;
      }
    }

    const info = await provider.connection.getAccountInfo(dispatcherStatePda);
    expect(info).to.not.be.null;
    const state = deserializeDispatcherState(info!.data);
    expect(state.authority.toString()).to.equal(authority.publicKey.toString());
    expect(state.registryProgramId.toString()).to.equal(
      PROGRAM_IDS.adapterRegistry.toString()
    );
    expect(state.totalDeposits).to.be.at.least(0);
    expect(state.isPaused).to.be.false;
  });

  it("deposits through the dispatcher via Kamino CPI", async () => {
    const setup = await setupApprovedKaminoForDispatcher(
      provider,
      payer,
      usdcMint,
    );

    const userTokenAccount = await fundUserForTest(usdcMint, 1_000_000);

    const positionPda = userPositionPda(
      PROGRAM_IDS.yieldDispatcher,
      authority.publicKey,
      setup.adapterProgram
    );

    const ix = buildDispatcherDeposit(
      {
        user: authority.publicKey,
        dispatcherState: dispatcherStatePda,
        userPosition: positionPda,
        registryProgram: PROGRAM_IDS.adapterRegistry,
        adapterEntry: setup.adapterEntryPda,
        adapterProgram: setup.adapterProgram,
        userTokenAccount,
        adapterVaultState: setup.vaultStatePda,
        adapterVault: setup.vaultTokenAccount,
        adapterVaultAuthority: setup.vaultAuthorityPda,
        adapterUserPosition: setup.adapterUserPositionPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      500_000,
    );
    await provider.sendIx(ix);

    const info = await provider.connection.getAccountInfo(positionPda);
    expect(info).to.not.be.null;
    const position = deserializeUserPosition(info!.data);
    expect(position.depositedAmount).to.equal(500_000);
    expect(position.receiptTokenBalance).to.be.greaterThan(0);

    const vaultBalance = await getTokenBalance(provider.connection, setup.vaultTokenAccount);
    expect(vaultBalance).to.be.at.least(500_000);
  });

  it("withdraws through the dispatcher via Kamino CPI", async () => {
    const setup = await setupApprovedKaminoForDispatcher(
      provider,
      payer,
      usdcMint,
    );

    const userTokenAccount = await fundUserForTest(usdcMint, 2_000_000);

    const positionPda = userPositionPda(
      PROGRAM_IDS.yieldDispatcher,
      authority.publicKey,
      setup.adapterProgram
    );

    const depositIx = buildDispatcherDeposit(
      {
        user: authority.publicKey,
        dispatcherState: dispatcherStatePda,
        userPosition: positionPda,
        registryProgram: PROGRAM_IDS.adapterRegistry,
        adapterEntry: setup.adapterEntryPda,
        adapterProgram: setup.adapterProgram,
        userTokenAccount,
        adapterVaultState: setup.vaultStatePda,
        adapterVault: setup.vaultTokenAccount,
        adapterVaultAuthority: setup.vaultAuthorityPda,
        adapterUserPosition: setup.adapterUserPositionPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      },
      1_000_000,
    );
    await provider.sendIx(depositIx);

    const beforeInfo = await provider.connection.getAccountInfo(positionPda);
    const beforePosition = deserializeUserPosition(beforeInfo!.data);
    const beforeWithdraw = beforePosition.receiptTokenBalance;

    const withdrawIx = buildDispatcherWithdraw(
      {
        user: authority.publicKey,
        dispatcherState: dispatcherStatePda,
        userPosition: positionPda,
        registryProgram: PROGRAM_IDS.adapterRegistry,
        adapterEntry: setup.adapterEntryPda,
        adapterProgram: setup.adapterProgram,
        userTokenAccount,
        adapterVaultState: setup.vaultStatePda,
        adapterVault: setup.vaultTokenAccount,
        adapterVaultAuthority: setup.vaultAuthorityPda,
        adapterUserPosition: setup.adapterUserPositionPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      400_000,
    );
    await provider.sendIx(withdrawIx);

    const info = await provider.connection.getAccountInfo(positionPda);
    const position = deserializeUserPosition(info!.data);
    expect(position.receiptTokenBalance).to.equal(
      beforeWithdraw - 400_000
    );

    const userBalance = await getTokenBalance(provider.connection, userTokenAccount);
    expect(userBalance).to.be.greaterThan(0);
  });

  it("rejects unapproved adapters", async () => {
    const unapprovedAdapter = Keypair.generate();
    const [adapterEntryPda] = findPda(
      [Buffer.from("adapter_entry"), unapprovedAdapter.publicKey.toBuffer()],
      PROGRAM_IDS.adapterRegistry
    );

    const proposeIx = buildProposeAdapter(
      {
        proposer: authority.publicKey,
        registryState: findPda([Buffer.from("registry_state")], PROGRAM_IDS.adapterRegistry)[0],
        adapterEntry: adapterEntryPda,
        adapterProgram: unapprovedAdapter.publicKey,
        underlyingMint: usdcMint,
        rent: RENT_SYSVAR,
        systemProgram: SystemProgram.programId,
      },
      "Fake",
      "https://example.com/fake.json",
    );
    await provider.sendIx(proposeIx);

    const kaminoSetup = await setupApprovedKaminoForDispatcher(
      provider,
      payer,
      usdcMint,
    );

    const userTokenAccount = await fundUserForTest(usdcMint, 1_000_000);

    const positionPda = userPositionPda(
      PROGRAM_IDS.yieldDispatcher,
      authority.publicKey,
      unapprovedAdapter.publicKey
    );

    try {
      const ix = buildDispatcherDeposit(
        {
          user: authority.publicKey,
          dispatcherState: dispatcherStatePda,
          userPosition: positionPda,
          registryProgram: PROGRAM_IDS.adapterRegistry,
          adapterEntry: adapterEntryPda,
          adapterProgram: unapprovedAdapter.publicKey,
          userTokenAccount,
          adapterVaultState: kaminoSetup.vaultStatePda,
          adapterVault: kaminoSetup.vaultTokenAccount,
          adapterVaultAuthority: kaminoSetup.vaultAuthorityPda,
          adapterUserPosition: kaminoSetup.adapterUserPositionPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        },
        500_000,
      );
      await provider.sendIx(ix);
      expect.fail("Should have failed to deposit to unapproved adapter");
    } catch (err: unknown) {
      expect(String(err)).to.contain("custom program error");
    }
  });

  it("rejects zero-amount deposits", async () => {
    const setup = await setupApprovedKaminoForDispatcher(
      provider,
      payer,
      usdcMint,
    );

    const userTokenAccount = await fundUserForTest(usdcMint, 1_000_000);

    const positionPda = userPositionPda(
      PROGRAM_IDS.yieldDispatcher,
      authority.publicKey,
      setup.adapterProgram
    );

    try {
      const ix = buildDispatcherDeposit(
        {
          user: authority.publicKey,
          dispatcherState: dispatcherStatePda,
          userPosition: positionPda,
          registryProgram: PROGRAM_IDS.adapterRegistry,
          adapterEntry: setup.adapterEntryPda,
          adapterProgram: setup.adapterProgram,
          userTokenAccount,
          adapterVaultState: setup.vaultStatePda,
          adapterVault: setup.vaultTokenAccount,
          adapterVaultAuthority: setup.vaultAuthorityPda,
          adapterUserPosition: setup.adapterUserPositionPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        },
        0,
      );
      await provider.sendIx(ix);
      expect.fail("Should have rejected zero deposit");
    } catch (err: unknown) {
      expect(String(err)).to.contain("custom program error");
    }
  });
});
