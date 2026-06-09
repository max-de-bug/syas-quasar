import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { createVaultTokenAccount, initializeAdapterVault } from "./adapter";
import { adapterUserPositionPda, findPda } from "./index";
import { TestProvider } from "./provider";
import {
  PROGRAM_IDS,
  TOKEN_PROGRAM_ID as TOKEN_PROGRAM,
} from "./constants";
import {
  RENT_SYSVAR,
  buildRegistryInitialize,
  buildProposeAdapter,
  buildApproveAdapter,
} from "./quasar-client";

export interface ApprovedAdapterSetup {
  adapterProgram: PublicKey;
  adapterEntryPda: PublicKey;
  vaultStatePda: PublicKey;
  vaultAuthorityPda: PublicKey;
  vaultTokenAccount: PublicKey;
  adapterUserPositionPda: PublicKey;
}

export async function ensureRegistryInitialized(
  provider: TestProvider,
): Promise<PublicKey> {
  const [registryStatePda] = findPda(
    [Buffer.from("registry_state")],
    PROGRAM_IDS.adapterRegistry
  );

  try {
    const ix = buildRegistryInitialize(
      provider.publicKey,
      registryStatePda,
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

  return registryStatePda;
}

export async function approveAdapterInRegistry(
  provider: TestProvider,
  registryStatePda: PublicKey,
  adapterProgram: PublicKey,
  underlyingMint: PublicKey,
  name: string,
  metadataUri: string
): Promise<PublicKey> {
  const [adapterEntryPda] = findPda(
    [Buffer.from("adapter_entry"), adapterProgram.toBuffer()],
    PROGRAM_IDS.adapterRegistry
  );

  const entryInfo = await provider.connection.getAccountInfo(adapterEntryPda);
  if (entryInfo) {
    return adapterEntryPda;
  }

  const proposeIx = buildProposeAdapter(
    {
      proposer: provider.publicKey,
      registryState: registryStatePda,
      adapterEntry: adapterEntryPda,
      adapterProgram,
      underlyingMint,
      rent: RENT_SYSVAR,
      systemProgram: SystemProgram.programId,
    },
    name,
    metadataUri,
  );
  await provider.sendIx(proposeIx);

  const approveIx = buildApproveAdapter(
    provider.publicKey,
    registryStatePda,
    adapterEntryPda,
  );
  await provider.sendIx(approveIx);

  return adapterEntryPda;
}

export async function setupReferenceAdapterVault(
  adapterProgramId: PublicKey,
  provider: TestProvider,
  payer: Keypair,
  underlyingMint: PublicKey,
  vaultStateSeed: string,
  vaultAuthoritySeed: string
): Promise<{
  vaultStatePda: PublicKey;
  vaultAuthorityPda: PublicKey;
  vaultTokenAccount: PublicKey;
}> {
  const [vaultStatePda] = findPda(
    [Buffer.from(vaultStateSeed)],
    adapterProgramId
  );
  const [vaultAuthorityPda] = findPda(
    [Buffer.from(vaultAuthoritySeed)],
    adapterProgramId
  );

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

  return { vaultStatePda, vaultAuthorityPda, vaultTokenAccount };
}

export async function resolveKaminoVaultMint(
  provider: TestProvider,
  fallbackMint: PublicKey
): Promise<PublicKey> {
  const [vaultStatePda] = findPda(
    [Buffer.from("kamino_vault_state")],
    PROGRAM_IDS.adapterKamino
  );

  try {
    const info = await provider.connection.getAccountInfo(vaultStatePda);
    if (info && info.data.length > 65) {
      return new PublicKey(info.data.subarray(33, 65));
    }
  } catch {
    // fallthrough
  }
  return fallbackMint;
}

export async function setupApprovedKaminoForDispatcher(
  provider: TestProvider,
  payer: Keypair,
  underlyingMint: PublicKey
): Promise<ApprovedAdapterSetup> {
  const kaminoProgramId = PROGRAM_IDS.adapterKamino;

  const registryStatePda = await ensureRegistryInitialized(provider);

  const adapterEntryPda = await approveAdapterInRegistry(
    provider,
    registryStatePda,
    kaminoProgramId,
    underlyingMint,
    "Kamino USDC (reference)",
    "https://example.com/kamino-reference.json"
  );

  const { vaultStatePda, vaultAuthorityPda, vaultTokenAccount } =
    await setupReferenceAdapterVault(
      kaminoProgramId,
      provider,
      payer,
      underlyingMint,
      "kamino_vault_state",
      "kamino_vault_authority"
    );

  return {
    adapterProgram: kaminoProgramId,
    adapterEntryPda,
    vaultStatePda,
    vaultAuthorityPda,
    vaultTokenAccount,
    adapterUserPositionPda: adapterUserPositionPda(
      provider.publicKey,
      kaminoProgramId
    ),
  };
}

export function userPositionPda(
  dispatcherProgramId: PublicKey,
  user: PublicKey,
  adapterProgram: PublicKey
): PublicKey {
  const [pda] = findPda(
    [
      Buffer.from("user_position"),
      user.toBuffer(),
      adapterProgram.toBuffer(),
    ],
    dispatcherProgramId
  );
  return pda;
}
