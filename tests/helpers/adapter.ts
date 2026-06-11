import {
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  transfer,
  getOrCreateAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";

import {
  adapterUserPositionPda,
  createTestMint,
  createTestTokenAccount,
  findPda,
  getTokenBalance,
  mintTestTokens,
} from "./index";
import {
  isMainnetFork,
  KAMINO_PROGRAM_ID,
  MARGINFI_PROGRAM_ID,
  DRIFT_PROGRAM_ID,
  JUPITER_PERPS_PROGRAM_ID,
  MAINNET_USDC_MINT,
  SYRUP_USDC_MINT,
  TOKEN_PROGRAM_ID as TOKEN_PROGRAM,
  PROGRAM_IDS,
} from "./constants";
import { TestProvider } from "./provider";
import {
  RENT_SYSVAR,
  buildAdapterInitialize,
  buildAdapterDeposit,
  buildAdapterWithdraw,
  buildAdapterCurrentValue,
} from "./quasar-client";
import * as fs from "fs";
import * as path from "path";

export interface AdapterTestContext {
  adapterProgramId: PublicKey;
  vaultStatePda: PublicKey;
  vaultAuthorityPda: PublicKey;
  vaultTokenAccount: PublicKey;
  underlyingMint: PublicKey;
  vaultStateSeed: string;
  vaultAuthoritySeed: string;
}

export interface AdapterFlowOptions {
  adapterProgramId: PublicKey;
  vaultStateSeed: string;
  vaultAuthoritySeed: string;
  depositAmount?: number;
  withdrawShares?: number;
  underlyingMint?: PublicKey;
}

function protocolProgramForAdapter(adapterProgramId: PublicKey): PublicKey | null {
  const id = adapterProgramId.toBase58();
  if (id === PROGRAM_IDS.adapterKamino.toBase58()) return KAMINO_PROGRAM_ID;
  if (id === PROGRAM_IDS.adapterMarginfi.toBase58()) return MARGINFI_PROGRAM_ID;
  if (id === PROGRAM_IDS.adapterJupiter.toBase58()) return JUPITER_PERPS_PROGRAM_ID;
  if (id === PROGRAM_IDS.adapterDrift.toBase58()) return DRIFT_PROGRAM_ID;
  return null;
}

export async function resolveUnderlyingMint(
  provider: TestProvider,
  payer: Keypair
): Promise<PublicKey> {
  if (isMainnetFork()) {
    return MAINNET_USDC_MINT;
  }
  return createTestMint(provider.connection, payer, 6);
}

async function fundUserTokenOnFork(
  provider: TestProvider,
  payer: Keypair,
  user: PublicKey,
  mint: PublicKey,
  fixtureFileName: string,
  setupScriptName: string,
  amount: number
): Promise<PublicKey> {
  const fixtureWalletPath = path.join(
    __dirname,
    "../fixtures/fork-wallet.json"
  );
  if (!fs.existsSync(fixtureWalletPath)) {
    throw new Error(
      `Missing ${fixtureWalletPath}. Run: ./scripts/setup-fork-usdc-fixture.sh`
    );
  }

  const fixtureSecret = Uint8Array.from(
    JSON.parse(fs.readFileSync(fixtureWalletPath, "utf8"))
  );
  const fixtureWallet = Keypair.fromSecretKey(fixtureSecret);

  const airdropSig = await provider.connection.requestAirdrop(
    fixtureWallet.publicKey,
    2 * 10_000_000_000
  );
  const latest = await provider.connection.getLatestBlockhash();
  await provider.connection.confirmTransaction({
    signature: airdropSig,
    ...latest,
  });

  const fixtureAta = getAssociatedTokenAddressSync(
    mint,
    fixtureWallet.publicKey
  );

  const fixtureInfo = await provider.connection.getAccountInfo(fixtureAta);
  if (!fixtureInfo) {
    throw new Error(
      `Fork fixture ATA ${fixtureAta.toBase58()} missing. Ensure ${setupScriptName} and run-mainnet-fork-tests.sh load ${fixtureFileName}`
    );
  }

  const userAta = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    payer,
    mint,
    user,
    false,
    undefined,
    undefined,
    TOKEN_PROGRAM
  );

  await getAccount(provider.connection, fixtureAta, undefined, TOKEN_PROGRAM);
  await getAccount(provider.connection, userAta.address, undefined, TOKEN_PROGRAM);

  await transfer(
    provider.connection,
    payer,
    fixtureAta,
    userAta.address,
    fixtureWallet,
    amount,
    [],
    undefined,
    TOKEN_PROGRAM
  );

  return userAta.address;
}

export async function fundUserUsdcOnFork(
  provider: TestProvider,
  payer: Keypair,
  user: PublicKey,
  amount: number
): Promise<PublicKey> {
  return fundUserTokenOnFork(
    provider, payer, user,
    MAINNET_USDC_MINT,
    "fork-usdc-ata.json",
    "setup-fork-usdc-fixture.sh",
    amount
  );
}

export async function fundUserSyrupUsdcOnFork(
  provider: TestProvider,
  payer: Keypair,
  user: PublicKey,
  amount: number
): Promise<PublicKey> {
  return fundUserTokenOnFork(
    provider, payer, user,
    SYRUP_USDC_MINT,
    "fork-syrup-usdc-ata.json",
    "setup-fork-syrup-usdc-fixture.sh",
    amount
  );
}

export async function assertProtocolProgramLoaded(
  connection: import("@solana/web3.js").Connection,
  programId: PublicKey,
  label: string
): Promise<void> {
  const info = await connection.getAccountInfo(programId);
  expect(info, `${label} program should exist on fork`).to.not.be.null;
  expect(info!.executable, `${label} should be executable`).to.be.true;
}

export async function initializeAdapterVault(
  provider: TestProvider,
  adapterProgramId: PublicKey,
  vaultStatePda: PublicKey,
  underlyingMint: PublicKey,
): Promise<void> {
  try {
    const ix = buildAdapterInitialize(
      adapterProgramId,
      provider.publicKey,
      vaultStatePda,
      underlyingMint,
      RENT_SYSVAR,
      SystemProgram.programId,
    );
    await provider.sendIx(ix);
  } catch (e: unknown) {
    const msg = String(e);
    if (
      !msg.includes("already in use") &&
      !msg.includes("0x0") &&
      !msg.includes("requires an uninitialized")
    ) {
      throw e;
    }
  }
}

export async function createVaultTokenAccount(
  provider: TestProvider,
  payer: Keypair,
  underlyingMint: PublicKey,
  vaultAuthorityPda: PublicKey
): Promise<PublicKey> {
  const account = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    payer,
    underlyingMint,
    vaultAuthorityPda,
    true
  );
  return account.address;
}

export async function runAdapterDepositWithdrawFlow(
  provider: TestProvider,
  payer: Keypair,
  options: AdapterFlowOptions
): Promise<void> {
  const {
    adapterProgramId,
    vaultStateSeed,
    vaultAuthoritySeed,
    depositAmount = 1_000_000,
    withdrawShares = 500_000,
    underlyingMint: explicitMint,
  } = options;

  const [vaultStatePda] = findPda(
    [Buffer.from(vaultStateSeed)],
    adapterProgramId
  );
  const [vaultAuthorityPda] = findPda(
    [Buffer.from(vaultAuthoritySeed)],
    adapterProgramId
  );

  // If vault already exists (e.g. created by dispatcher tests), use its mint
  let underlyingMint: PublicKey;
  const vaultInfo = await provider.connection.getAccountInfo(vaultStatePda);
  if (vaultInfo) {
    underlyingMint = new PublicKey(vaultInfo.data.subarray(33, 65));
  } else if (explicitMint) {
    underlyingMint = explicitMint;
  } else {
    underlyingMint = await resolveUnderlyingMint(provider, payer);
  }

  await initializeAdapterVault(
    provider,
    adapterProgramId,
    vaultStatePda,
    underlyingMint,
  );

  const vaultTokenAccount = await createVaultTokenAccount(
    provider,
    payer,
    underlyingMint,
    vaultAuthorityPda
  );

  let userTokenAccount: PublicKey;
  if (isMainnetFork()) {
    if (explicitMint && explicitMint.equals(SYRUP_USDC_MINT)) {
      userTokenAccount = await fundUserSyrupUsdcOnFork(
        provider,
        payer,
        provider.publicKey,
        depositAmount * 2
      );
    } else {
      userTokenAccount = await fundUserUsdcOnFork(
        provider,
        payer,
        provider.publicKey,
        depositAmount * 2
      );
    }
  } else {
    userTokenAccount = await createTestTokenAccount(
      provider.connection,
      underlyingMint,
      provider.publicKey,
      payer
    );
    await mintTestTokens(
      provider.connection,
      underlyingMint,
      userTokenAccount,
      payer,
      depositAmount * 2
    );
  }

  const userPositionPda = adapterUserPositionPda(
    provider.publicKey,
    adapterProgramId
  );

  const depositAccounts = {
    user: provider.publicKey,
    vaultState: vaultStatePda,
    userPosition: userPositionPda,
    userTokenAccount,
    vaultAuthority: vaultAuthorityPda,
    vaultTokenAccount,
    tokenProgram: TOKEN_PROGRAM,
    systemProgram: SystemProgram.programId,
  };

  const vaultBalanceBeforeDeposit = await getTokenBalance(
    provider.connection,
    vaultTokenAccount
  );

  const depositIx = buildAdapterDeposit(
    adapterProgramId,
    depositAccounts,
    depositAmount,
    undefined,
  );
  await provider.sendIx(depositIx);

  const vaultBalanceAfterDeposit = await getTokenBalance(
    provider.connection,
    vaultTokenAccount
  );
  expect(vaultBalanceAfterDeposit - vaultBalanceBeforeDeposit).to.equal(depositAmount);

  const currentValueIx = buildAdapterCurrentValue(
    adapterProgramId,
    {
      user: provider.publicKey,
      vaultState: vaultStatePda,
      userPosition: userPositionPda,
    },
    undefined,
  );
  await provider.sendIx(currentValueIx);

  const vaultBalanceBeforeWithdraw = await getTokenBalance(
    provider.connection,
    vaultTokenAccount
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
      tokenProgram: TOKEN_PROGRAM,
    },
    withdrawShares,
    undefined,
  );
  await provider.sendIx(withdrawIx);

  const userBalance = await getTokenBalance(provider.connection, userTokenAccount);
  expect(userBalance).to.be.greaterThan(0);

  const vaultBalanceAfterWithdraw = await getTokenBalance(
    provider.connection,
    vaultTokenAccount
  );
  expect(vaultBalanceAfterWithdraw).to.be.lessThan(vaultBalanceBeforeWithdraw);
}
