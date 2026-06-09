import { Keypair } from "@solana/web3.js";

import {
  assertProtocolProgramLoaded,
  runAdapterDepositWithdrawFlow,
} from "../helpers/adapter";
import { TestProvider } from "../helpers/provider";
import { isMainnetFork, MARGINFI_PROGRAM_ID, PROGRAM_IDS } from "../helpers/constants";

describe("adapter-marginfi", () => {
  const provider = TestProvider.env();
  const payer = provider.wallet as Keypair;

  if (isMainnetFork()) {
    it("loads MarginFi v2 program from mainnet fork", async () => {
      await assertProtocolProgramLoaded(
        provider.connection,
        MARGINFI_PROGRAM_ID,
        "MarginFi v2"
      );
    });
  }

  it("deposit → current_value → withdraw", async () => {
    await runAdapterDepositWithdrawFlow(provider, payer, {
      adapterProgramId: PROGRAM_IDS.adapterMarginfi,
      vaultStateSeed: "marginfi_vault_state",
      vaultAuthoritySeed: "marginfi_vault_authority",
    });
  });
});
