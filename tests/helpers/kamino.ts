import { AccountMeta, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID as TOKEN_PROGRAM } from "./constants";

/**
 * Kamino fork test account constants
 * These are cloned from mainnet during fork tests
 */
export const KAMINO_FORK_ACCOUNTS = {
  program: new PublicKey("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"),
  lendingMarket: new PublicKey("7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF"),
  usdcReserve: new PublicKey("d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q"),
  
  // Reserve-derived accounts for USDC reserve
  // (These would normally be extracted by deserializing the reserve account)
  marketAuthority: new PublicKey("DhWcnsGmKGuyrgnij5o4G9VXJ72EhCzen9MuHqNAiXkN"),
  liquidityMint: new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC
  liquiditySupply: new PublicKey("BcnvxFj7xr7N3W8nBseFfhCVMKu7VRhUXvhpFtXXyqN7"),
  collateralMint: new PublicKey("kToken2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD"), // Kamino kUSDC
  
  // Will be derived per test: vault's collateral token account
  // (Need to create this as an ATA of the vault authority in the test)
};

/**
 * Remaining accounts for Kamino deposit CPI (11 accounts total)
 * Order must match adapter-kamino/src/protocol.rs on_deposit()
 */
export function buildKaminoDepositRemainingAccounts(
  vaultCollateralTokenAccount: PublicKey
): AccountMeta[] {
  return [
    // 0: Kamino K-Lend program
    { pubkey: KAMINO_FORK_ACCOUNTS.program, isSigner: false, isWritable: false },
    // 1: reserve (writable)
    { pubkey: KAMINO_FORK_ACCOUNTS.usdcReserve, isSigner: false, isWritable: true },
    // 2: lending_market
    { pubkey: KAMINO_FORK_ACCOUNTS.lendingMarket, isSigner: false, isWritable: false },
    // 3: market_authority
    { pubkey: KAMINO_FORK_ACCOUNTS.marketAuthority, isSigner: false, isWritable: false },
    // 4: liq_mint (USDC)
    { pubkey: KAMINO_FORK_ACCOUNTS.liquidityMint, isSigner: false, isWritable: false },
    // 5: liq_supply (writable)
    { pubkey: KAMINO_FORK_ACCOUNTS.liquiditySupply, isSigner: false, isWritable: true },
    // 6: collat_mint (kUSDC)
    { pubkey: KAMINO_FORK_ACCOUNTS.collateralMint, isSigner: false, isWritable: false },
    // 7: vault_ctoken (writable) - vault's collateral token account
    { pubkey: vaultCollateralTokenAccount, isSigner: false, isWritable: true },
    // 8: collat_token_prog (SPL token program)
    { pubkey: TOKEN_PROGRAM, isSigner: false, isWritable: false },
    // 9: sysvar_ix (instruction sysvar)
    { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
  ];
}

/**
 * Remaining accounts for Kamino withdraw CPI (10 accounts total)
 * Order must match adapter-kamino/src/protocol.rs on_withdraw()
 * NOTE: Order differs from deposit in Kamino's actual CPI
 */
export function buildKaminoWithdrawRemainingAccounts(
  vaultCollateralTokenAccount: PublicKey
): AccountMeta[] {
  return [
    // 0: Kamino K-Lend program
    { pubkey: KAMINO_FORK_ACCOUNTS.program, isSigner: false, isWritable: false },
    // 1: reserve (writable)
    { pubkey: KAMINO_FORK_ACCOUNTS.usdcReserve, isSigner: false, isWritable: true },
    // 2: lending_market
    { pubkey: KAMINO_FORK_ACCOUNTS.lendingMarket, isSigner: false, isWritable: false },
    // 3: market_authority
    { pubkey: KAMINO_FORK_ACCOUNTS.marketAuthority, isSigner: false, isWritable: false },
    // 4: liq_mint (USDC)
    { pubkey: KAMINO_FORK_ACCOUNTS.liquidityMint, isSigner: false, isWritable: false },
    // 5: collat_mint (kUSDC)
    { pubkey: KAMINO_FORK_ACCOUNTS.collateralMint, isSigner: false, isWritable: false },
    // 6: liq_supply (writable)
    { pubkey: KAMINO_FORK_ACCOUNTS.liquiditySupply, isSigner: false, isWritable: true },
    // 7: vault_ctoken (writable)
    { pubkey: vaultCollateralTokenAccount, isSigner: false, isWritable: true },
    // 8: collat_token_prog (SPL token program)
    { pubkey: TOKEN_PROGRAM, isSigner: false, isWritable: false },
    // 9: sysvar_ix (instruction sysvar)
    { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
  ];
}
