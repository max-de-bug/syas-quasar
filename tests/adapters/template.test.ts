/*
 * ─── TEMPLATE ADAPTER TEST ─────────────────────────────────────────────────────
 *
 * This test follows the same pattern as all other adapter tests (kamino, marginfi,
 * jupiter, maple, drift). It verifies the full deposit → current_value → withdraw
 * lifecycle on both localnet and mainnet fork.
 *
 * When copying this adapter to create a real adapter:
 *   1. Rename this file to match your program name (e.g., adapter-mysolana.test.ts)
 *   2. Change `PROGRAM_IDS.adapterTemplate` to your program's ID constant
 *   3. Update vaultStateSeed and vaultAuthoritySeed to match your protocol
 *   4. Add a mainnet program ID assertion in the `if (isMainnetFork())` block
 */

import { Keypair } from "@solana/web3.js";

import {
  assertProtocolProgramLoaded,
  runAdapterDepositWithdrawFlow,
} from "../helpers/adapter";
import { TestProvider } from "../helpers/provider";
import { isMainnetFork, PROGRAM_IDS } from "../helpers/constants";

describe("adapter-template", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  // In a real adapter, add a fork-verification test like:
  // if (isMainnetFork()) {
  //   it("loads MyProtocol program from mainnet fork", async () => {
  //     await assertProtocolProgramLoaded(
  //       provider.connection,
  //       MY_PROTOCOL_ID,
  //       "My Protocol"
  //     );
  //   });
  // }

  it("deposit → current_value → withdraw", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterTemplate,
      vaultStateSeed: "template_vault_state",
      vaultAuthoritySeed: "template_vault_authority",
    });
  });
});
