import { Keypair } from "@solana/web3.js";
import { expect } from "chai";

import {
  assertProtocolProgramLoaded,
  runAdapterDepositWithdrawFlow,
} from "../helpers/adapter";
import { TestProvider } from "../helpers/provider";
import { isMainnetFork, JUPITER_PERPS_PROGRAM_ID, PROGRAM_IDS } from "../helpers/constants";

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
});
