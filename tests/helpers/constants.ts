import { PublicKey } from "@solana/web3.js";

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

export const MAINNET_USDC_MINT = new PublicKey(
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
);

export const SYRUP_USDC_MINT = new PublicKey(
  "AvZZF1YaZDziPY2RCK4oJrRVrbN3mTD9NL24hPeaZeUj"
);

export const KAMINO_PROGRAM_ID = new PublicKey(
  "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"
);
export const MARGINFI_PROGRAM_ID = new PublicKey(
  "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA"
);
export const DRIFT_PROGRAM_ID = new PublicKey(
  "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH"
);
export const JUPITER_PERPS_PROGRAM_ID = new PublicKey(
  "PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu"
);

export const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

export function isMainnetFork(): boolean {
  return process.env.MAINNET_FORK === "1";
}
