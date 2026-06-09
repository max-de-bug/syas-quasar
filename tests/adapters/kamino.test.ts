import { Keypair } from "@solana/web3.js";

import {
  assertProtocolProgramLoaded,
  runAdapterDepositWithdrawFlow,
} from "../helpers/adapter";
import { TestProvider } from "../helpers/provider";
import { isMainnetFork, KAMINO_PROGRAM_ID, PROGRAM_IDS } from "../helpers/constants";

describe("adapter-kamino", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  if (isMainnetFork()) {
    it("loads Kamino K-Lend program from mainnet fork", async () => {
      await assertProtocolProgramLoaded(
        provider.connection,
        KAMINO_PROGRAM_ID,
        "Kamino K-Lend"
      );
    });
  }

  it("deposit → current_value → withdraw", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterKamino,
      vaultStateSeed: "kamino_vault_state",
      vaultAuthoritySeed: "kamino_vault_authority",
    });
  });
});
