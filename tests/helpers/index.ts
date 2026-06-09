import {
  Connection,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  mintTo,
  getAccount,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { expect } from "chai";

export async function airdrop(
  connection: Connection,
  to: PublicKey,
  amount: number = 10 * LAMPORTS_PER_SOL
): Promise<void> {
  const sig = await connection.requestAirdrop(to, amount);
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature: sig,
    ...latestBlockhash,
  });
}

export async function createTestMint(
  connection: Connection,
  authority: Keypair,
  decimals: number = 6
): Promise<PublicKey> {
  return createMint(connection, authority, authority.publicKey, null, decimals);
}

export async function createTestTokenAccount(
  connection: Connection,
  mint: PublicKey,
  owner: PublicKey,
  payer: Keypair
): Promise<PublicKey> {
  const account = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    owner
  );
  return account.address;
}

export async function mintTestTokens(
  connection: Connection,
  mint: PublicKey,
  destination: PublicKey,
  authority: Keypair,
  amount: number
): Promise<void> {
  await mintTo(connection, authority, mint, destination, authority, amount);
}

export async function getTokenBalance(
  connection: Connection,
  tokenAccount: PublicKey
): Promise<number> {
  const account = await getAccount(connection, tokenAccount);
  return Number(account.amount);
}

export function findPda(
  seeds: Buffer[],
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(seeds, programId);
}

export function adapterUserPositionPda(
  user: PublicKey,
  adapterProgramId: PublicKey
): PublicKey {
  const [pda] = findPda(
    [Buffer.from("adapter_position"), user.toBuffer()],
    adapterProgramId
  );
  return pda;
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
