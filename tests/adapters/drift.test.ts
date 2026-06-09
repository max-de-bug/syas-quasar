import { Keypair } from "@solana/web3.js";

import {
  assertProtocolProgramLoaded,
  runAdapterDepositWithdrawFlow,
} from "../helpers/adapter";
import { TestProvider } from "../helpers/provider";
import { DRIFT_PROGRAM_ID, isMainnetFork, PROGRAM_IDS } from "../helpers/constants";

describe("adapter-drift", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  if (isMainnetFork()) {
    it("loads Drift v2 program from mainnet fork", async () => {
      await assertProtocolProgramLoaded(
        provider.connection,
        DRIFT_PROGRAM_ID,
        "Drift v2"
      );
    });
  }

  it("deposit → current_value → withdraw (insurance fund model)", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterDrift,
      vaultStateSeed: "drift_vault_state",
      vaultAuthoritySeed: "drift_vault_authority",
      withdrawShares: 500_000,
    });
  });
});
