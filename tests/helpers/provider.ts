import {
  Commitment,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
  TransactionInstruction,
} from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";

export class TestProvider {
  readonly connection: Connection;
  readonly wallet: Keypair;

  constructor(connection: Connection, wallet: Keypair) {
    this.connection = connection;
    this.wallet = wallet;
  }

  get publicKey(): PublicKey {
    return this.wallet.publicKey;
  }

  static env(): TestProvider {
    const url =
      process.env.ANCHOR_PROVIDER_URL || "http://localhost:8899";
    const walletPath =
      process.env.ANCHOR_WALLET ||
      path.join(os.homedir(), ".config/solana/id.json");
    const secret = JSON.parse(
      fs.readFileSync(walletPath.replace(/^~/, os.homedir()), "utf8")
    );
    const wallet = Keypair.fromSecretKey(Uint8Array.from(secret));
    const connection = new Connection(url, "confirmed" as Commitment);
    return new TestProvider(connection, wallet);
  }

  async sendIx(
    instruction: TransactionInstruction,
    signers?: Keypair[]
  ): Promise<string> {
    const tx = new Transaction().add(instruction);
    const allSigners = [this.wallet, ...(signers || [])];
    const sig = await sendAndConfirmTransaction(
      this.connection,
      tx,
      allSigners
    );
    return sig;
  }

  async sendIxs(
    instructions: TransactionInstruction[],
    signers?: Keypair[]
  ): Promise<string> {
    const tx = new Transaction();
    for (const ix of instructions) {
      tx.add(ix);
    }
    const allSigners = [this.wallet, ...(signers || [])];
    const sig = await sendAndConfirmTransaction(
      this.connection,
      tx,
      allSigners
    );
    return sig;
  }
}
