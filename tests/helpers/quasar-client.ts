import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";

export const RENT_SYSVAR = SYSVAR_RENT_PUBKEY;

export const PROGRAM_IDS = {
  adapterRegistry: new PublicKey("CeyDkRgegNUz2TeFfFjRdL89G9EGGDymiqHoJkeFGcZ4"),
  adapterKamino: new PublicKey("BzuVWb3UgCW6axee6ZNb812D268XrWkJsE7mxkX9b3Kp"),
  adapterMarginfi: new PublicKey("FrCvyyGSukMZcLhpU7EneuhfPmqS5p8E2ysnFdwHhopR"),
  adapterJupiter: new PublicKey("2acqkTDi2VQ4FCZVDB8PeMVLVWnREogE5HA2GxvHdWxu"),
  adapterMaple: new PublicKey("Ft2Yvaiqwsjvo1yyYEWvt12YCsDB4kjGBd7vrF8RwwjU"),
  adapterDrift: new PublicKey("CVfb8T9tf9WEeus4mKWsxTehVezeY9TGwYsSc3JmxWYz"),
  adapterTemplate: new PublicKey("AzGucBSAxRMme758P9WsXqZASqnea7xZqKr7ys6gvCcX"),
  yieldDispatcher: new PublicKey("7oUKys5XKMzD2NmFCZyLDyTF2Hm1VH3qX8jVfZEY4f3r"),
};

function encodeU64(value: number): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(value), 0);
  return buf;
}

function encodePubkey(pk: PublicKey): Buffer {
  return Buffer.from(pk.toBytes());
}

function encodeString(s: string): Buffer {
  const encoder = new TextEncoder();
  const bytes = Buffer.from(encoder.encode(s));
  const len = Buffer.alloc(2);
  len.writeUInt16LE(bytes.length, 0);
  return Buffer.concat([len, bytes]);
}

// ── Registry instructions ──────────────────────────────────────

export function buildRegistryInitialize(
  authority: PublicKey,
  registryState: PublicKey,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.adapterRegistry,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: RENT_SYSVAR, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([0]),
  });
}

export function buildProposeAdapter(
  accounts: {
    proposer: PublicKey;
    registryState: PublicKey;
    adapterEntry: PublicKey;
    adapterProgram: PublicKey;
    underlyingMint: PublicKey;
    rent: PublicKey;
    systemProgram: PublicKey;
  },
  name: string,
  metadataUri: string,
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from([1]),
    encodeString(name),
    encodeString(metadataUri),
  ]);
  return new TransactionInstruction({
    programId: PROGRAM_IDS.adapterRegistry,
    keys: [
      { pubkey: accounts.proposer, isSigner: true, isWritable: true },
      { pubkey: accounts.registryState, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterEntry, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.underlyingMint, isSigner: false, isWritable: false },
      { pubkey: accounts.rent, isSigner: false, isWritable: false },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function buildApproveAdapter(
  authority: PublicKey,
  registryState: PublicKey,
  adapterEntry: PublicKey,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.adapterRegistry,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: adapterEntry, isSigner: false, isWritable: true },
    ],
    data: Buffer.from([2]),
  });
}

export function buildRevokeAdapter(
  authority: PublicKey,
  registryState: PublicKey,
  adapterEntry: PublicKey,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.adapterRegistry,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: adapterEntry, isSigner: false, isWritable: true },
    ],
    data: Buffer.from([3]),
  });
}

export function buildTransferGovernance(
  authority: PublicKey,
  registryState: PublicKey,
  newAuthority: PublicKey,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.adapterRegistry,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: registryState, isSigner: false, isWritable: true },
      { pubkey: newAuthority, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([4]),
  });
}

// ── Adapter vault instructions (generic for all 5 adapters) ─

export function buildAdapterInitialize(
  adapterProgramId: PublicKey,
  authority: PublicKey,
  vaultState: PublicKey,
  underlyingMint: PublicKey,
  rent: PublicKey,
  systemProgram: PublicKey,
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from([0]),
    encodePubkey(underlyingMint),
  ]);
  return new TransactionInstruction({
    programId: adapterProgramId,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: vaultState, isSigner: false, isWritable: true },
      { pubkey: rent, isSigner: false, isWritable: false },
      { pubkey: systemProgram, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function buildAdapterDeposit(
  adapterProgramId: PublicKey,
  accounts: {
    user: PublicKey;
    vaultState: PublicKey;
    userPosition: PublicKey;
    userTokenAccount: PublicKey;
    vaultAuthority: PublicKey;
    vaultTokenAccount: PublicKey;
    tokenProgram: PublicKey;
    systemProgram: PublicKey;
  },
  amount: number,
  remainingAccounts?: AccountMeta[],
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.user, isSigner: true, isWritable: true },
    { pubkey: accounts.vaultState, isSigner: false, isWritable: true },
    { pubkey: accounts.userPosition, isSigner: false, isWritable: true },
    { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
    { pubkey: accounts.vaultAuthority, isSigner: false, isWritable: false },
    { pubkey: accounts.vaultTokenAccount, isSigner: false, isWritable: true },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
  ];
  if (remainingAccounts) {
    keys.push(...remainingAccounts);
  }
  const data = Buffer.concat([Buffer.from([1]), encodeU64(amount)]);
  return new TransactionInstruction({
    programId: adapterProgramId,
    keys,
    data,
  });
}

export function buildAdapterWithdraw(
  adapterProgramId: PublicKey,
  accounts: {
    user: PublicKey;
    vaultState: PublicKey;
    userPosition: PublicKey;
    userTokenAccount: PublicKey;
    vaultTokenAccount: PublicKey;
    vaultAuthority: PublicKey;
    tokenProgram: PublicKey;
  },
  shares: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: adapterProgramId,
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: true },
      { pubkey: accounts.vaultState, isSigner: false, isWritable: true },
      { pubkey: accounts.userPosition, isSigner: false, isWritable: true },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.vaultTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([Buffer.from([2]), encodeU64(shares)]),
  });
}

export function buildAdapterCurrentValue(
  adapterProgramId: PublicKey,
  accounts: {
    user: PublicKey;
    vaultState: PublicKey;
    userPosition: PublicKey;
  },
  remainingAccounts?: AccountMeta[],
): TransactionInstruction {
  const keys: AccountMeta[] = [
    { pubkey: accounts.user, isSigner: true, isWritable: false },
    { pubkey: accounts.vaultState, isSigner: false, isWritable: true },
    { pubkey: accounts.userPosition, isSigner: false, isWritable: false },
  ];
  if (remainingAccounts) {
    keys.push(...remainingAccounts);
  }
  return new TransactionInstruction({
    programId: adapterProgramId,
    keys,
    data: Buffer.from([3]),
  });
}

// ── Dispatcher instructions ─────────────────────────────────

export function buildDispatcherInitialize(
  authority: PublicKey,
  dispatcherState: PublicKey,
  registryProgram: PublicKey,
  rent: PublicKey,
  systemProgram: PublicKey,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.yieldDispatcher,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: dispatcherState, isSigner: false, isWritable: true },
      { pubkey: registryProgram, isSigner: false, isWritable: false },
      { pubkey: rent, isSigner: false, isWritable: false },
      { pubkey: systemProgram, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([0]),
  });
}

export function buildDispatcherDeposit(
  accounts: {
    user: PublicKey;
    dispatcherState: PublicKey;
    userPosition: PublicKey;
    registryProgram: PublicKey;
    adapterEntry: PublicKey;
    adapterProgram: PublicKey;
    userTokenAccount: PublicKey;
    adapterVaultState: PublicKey;
    adapterVault: PublicKey;
    adapterVaultAuthority: PublicKey;
    adapterUserPosition: PublicKey;
    tokenProgram: PublicKey;
    systemProgram: PublicKey;
  },
  amount: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.yieldDispatcher,
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: true },
      { pubkey: accounts.dispatcherState, isSigner: false, isWritable: true },
      { pubkey: accounts.userPosition, isSigner: false, isWritable: true },
      { pubkey: accounts.registryProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterEntry, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterVaultState, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterVault, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterVaultAuthority, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterUserPosition, isSigner: false, isWritable: true },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([Buffer.from([1]), encodeU64(amount)]),
  });
}

export function buildDispatcherWithdraw(
  accounts: {
    user: PublicKey;
    dispatcherState: PublicKey;
    userPosition: PublicKey;
    registryProgram: PublicKey;
    adapterEntry: PublicKey;
    adapterProgram: PublicKey;
    userTokenAccount: PublicKey;
    adapterVaultState: PublicKey;
    adapterVault: PublicKey;
    adapterVaultAuthority: PublicKey;
    adapterUserPosition: PublicKey;
    tokenProgram: PublicKey;
  },
  shares: number,
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.yieldDispatcher,
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: true },
      { pubkey: accounts.dispatcherState, isSigner: false, isWritable: true },
      { pubkey: accounts.userPosition, isSigner: false, isWritable: true },
      { pubkey: accounts.registryProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterEntry, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterVaultState, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterVault, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterVaultAuthority, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterUserPosition, isSigner: false, isWritable: true },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    ],
    data: Buffer.concat([Buffer.from([2]), encodeU64(shares)]),
  });
}

export function buildDispatcherCurrentValue(
  accounts: {
    user: PublicKey;
    dispatcherState: PublicKey;
    userPosition: PublicKey;
    registryProgram: PublicKey;
    adapterEntry: PublicKey;
    adapterProgram: PublicKey;
    adapterVaultState: PublicKey;
    adapterUserPosition: PublicKey;
  },
): TransactionInstruction {
  return new TransactionInstruction({
    programId: PROGRAM_IDS.yieldDispatcher,
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: false },
      { pubkey: accounts.dispatcherState, isSigner: false, isWritable: false },
      { pubkey: accounts.userPosition, isSigner: false, isWritable: false },
      { pubkey: accounts.registryProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterEntry, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.adapterVaultState, isSigner: false, isWritable: true },
      { pubkey: accounts.adapterUserPosition, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([3]),
  });
}

// ── Account deserialization ──────────────────────────────────

function readPubkey(data: Buffer, offset: number): PublicKey {
  return new PublicKey(data.subarray(offset, offset + 32));
}

function readU64(data: Buffer, offset: number): number {
  return Number(data.readBigUInt64LE(offset));
}

function readI64(data: Buffer, offset: number): number {
  return Number(data.readBigInt64LE(offset));
}

function readString(data: Buffer, offset: number): { value: string; nextOffset: number } {
  const len = data.readUInt32LE(offset);
  const str = data.toString("utf8", offset + 4, offset + 4 + len);
  return { value: str, nextOffset: offset + 4 + len };
}


export interface DeserializedRegistryState {
  authority: PublicKey;
  totalProposed: number;
  totalApproved: number;
}

export function deserializeRegistryState(data: Buffer): DeserializedRegistryState {
  const disc = data[0];
  if (disc !== 1) {
    throw new Error(`Expected RegistryState discriminator 1, got ${disc}`);
  }
  const authority = readPubkey(data, 1);
  const totalOff = 66;
  const totalProposed = readU64(data, totalOff);
  const totalApproved = readU64(data, totalOff + 8);
  return { authority, totalProposed, totalApproved };
}

export interface DeserializedAdapterEntry {
  adapterProgramId: PublicKey;
  status: number;
  underlyingMint: PublicKey;
  proposer: PublicKey;
  proposedAt: number;
  approvedAt: number;
  revokedAt: number;
  name: string;
  metadataUri: string;
}

export function deserializeAdapterEntry(data: Buffer): DeserializedAdapterEntry {
  const disc = data[0];
  if (disc !== 2) {
    throw new Error(`Expected AdapterEntry discriminator 2, got ${disc}`);
  }
  const adapterProgramId = readPubkey(data, 1);
  const status = data[33];
  const underlyingMint = readPubkey(data, 34);
  const proposer = readPubkey(data, 66);
  const proposedAt = readI64(data, 98);
  const approvedAt = readI64(data, 106);
  const revokedAt = readI64(data, 114);
  const bump = data[122];
  const nameLen = data[123];
  const metaLen = data[156];
  const name = data.toString("utf8", 124, 124 + nameLen);
  const metadataUri = data.toString("utf8", 157, 157 + metaLen);
  return {
    adapterProgramId,
    status,
    underlyingMint,
    proposer,
    proposedAt,
    approvedAt,
    revokedAt,
    name,
    metadataUri,
  };
}

export interface DeserializedDispatcherState {
  authority: PublicKey;
  registryProgramId: PublicKey;
  totalDeposits: number;
  totalWithdrawals: number;
  isPaused: boolean;
}

export function deserializeDispatcherState(data: Buffer): DeserializedDispatcherState {
  const disc = data[0];
  if (disc !== 1) {
    throw new Error(`Expected DispatcherState discriminator 1, got ${disc}`);
  }
  const authority = readPubkey(data, 1);
  const registryProgramId = readPubkey(data, 33);
  const totalDeposits = readU64(data, 65);
  const totalWithdrawals = readU64(data, 73);
  const isPaused = data[81] !== 0;
  return { authority, registryProgramId, totalDeposits, totalWithdrawals, isPaused };
}

export interface DeserializedUserPosition {
  owner: PublicKey;
  adapterProgramId: PublicKey;
  depositedAmount: number;
  withdrawnAmount: number;
  receiptTokenBalance: number;
  lastUpdated: number;
}

export function deserializeUserPosition(data: Buffer): DeserializedUserPosition {
  const disc = data[0];
  if (disc !== 2) {
    throw new Error(`Expected UserPosition discriminator 2, got ${disc}`);
  }
  const owner = readPubkey(data, 1);
  const adapterProgramId = readPubkey(data, 33);
  const depositedAmount = readU64(data, 65);
  const withdrawnAmount = readU64(data, 73);
  const receiptTokenBalance = readU64(data, 81);
  const lastUpdated = readI64(data, 89);
  return { owner, adapterProgramId, depositedAmount, withdrawnAmount, receiptTokenBalance, lastUpdated };
}

export interface DeserializedVaultState {
  authority: PublicKey;
  underlyingMint: PublicKey;
  totalUnderlying: number;
  totalShares: number;
  protocolProgramId: PublicKey;
  protocolRoutedUnderlying: number;
  lastYieldSyncTs: number;
  isActive: boolean;
}

export function deserializeVaultState(data: Buffer): DeserializedVaultState {
  const disc = data[0];
  if (disc !== 1) {
    throw new Error(`Expected VaultState discriminator 1, got ${disc}`);
  }
  const authority = readPubkey(data, 1);
  const underlyingMint = readPubkey(data, 33);
  const totalUnderlying = readU64(data, 65);
  const totalShares = readU64(data, 73);
  const protocolProgramId = readPubkey(data, 81);
  const protocolRoutedUnderlying = readU64(data, 113);
  const lastYieldSyncTs = readI64(data, 121);
  const isActive = data[129] !== 0;
  return { authority, underlyingMint, totalUnderlying, totalShares, protocolProgramId, protocolRoutedUnderlying, lastYieldSyncTs, isActive };
}
