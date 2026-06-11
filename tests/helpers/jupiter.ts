import { AccountMeta, PublicKey, SYSVAR_INSTRUCTIONS_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID as TOKEN_PROGRAM } from "./constants";

/**
 * Jupiter fork test account constants
 * These should be cloned from mainnet during fork tests
 * Note: Some of these may need to be added to run-mainnet-fork-tests.sh
 */
export const JUPITER_FORK_ACCOUNTS = {
  program: new PublicKey("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu"),
  
  // Perpetuals state/market accounts
  perpetuals: new PublicKey("98pjRUQjBVCQdJbxF9c6ff6d4W1P5u7YBVT2XWsBXxA"), // Needs verification
  
  // USDC pool/market
  pool: new PublicKey("Cn6ABSTYLZn4BsUzHgxpUpFqrHzWvemZnKanYVhm7c9"), // Needs verification
  
  // USDC custody (already cloned in fork test script)
  usdcCustody: new PublicKey("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa"),
  
  // Doves price oracle (for oracle feeds)
  dovesPrice: new PublicKey("98pjRUQjBVCQdJbxF9c6ff6d4W1P5u7YBVT2XWsBXxA"), // Needs verification
  
  // Pyth price oracle
  pythPrice: new PublicKey("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD"), // USDC/USD Pyth feed
  
  // USDC token custody account (SPL token account holding USDC for the pool)
  usdcTokenCustody: new PublicKey("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa"),
  
  // LP token mint (JLP or similar)
  lpTokenMint: new PublicKey("98pjRUQjBVCQdJbxF9c6ff6d4W1P5u7YBVT2XWsBXxA"), // Needs verification
  
  // Event authority
  eventAuthority: new PublicKey("98pjRUQjBVCQdJbxF9c6ff6d4W1P5u7YBVT2XWsBXxA"), // Needs verification
};

/**
 * Remaining accounts for Jupiter deposit CPI (add_liquidity2) - 11 accounts total
 * Note: Some account addresses need verification from Jupiter's actual on-chain state
 * 
 * Account order from protocol.rs on_deposit():
 * 0: program
 * 1: transfer_authority
 * 2: perpetuals
 * 3: pool
 * 4: custody
 * 5: custody_doves_price
 * 6: custody_pyth_price
 * 7: custody_token_account
 * 8: lp_token_mint
 * 9: event_authority
 * 10: vault_lp (vault's LP token account - provided separately)
 */
export function buildJupiterDepositRemainingAccounts(
  transferAuthority: PublicKey,
  vaultLpTokenAccount: PublicKey
): AccountMeta[] {
  return [
    // 0: Jupiter Perp program
    { pubkey: JUPITER_FORK_ACCOUNTS.program, isSigner: false, isWritable: false },
    // 1: transfer_authority (signer for liquidity approval)
    { pubkey: transferAuthority, isSigner: false, isWritable: false },
    // 2: perpetuals (Perpetuals state)
    { pubkey: JUPITER_FORK_ACCOUNTS.perpetuals, isSigner: false, isWritable: false },
    // 3: pool (writable) - USDC pool
    { pubkey: JUPITER_FORK_ACCOUNTS.pool, isSigner: false, isWritable: true },
    // 4: custody (writable) - USDC custody
    { pubkey: JUPITER_FORK_ACCOUNTS.usdcCustody, isSigner: false, isWritable: true },
    // 5: custody_doves_price - Doves oracle price
    { pubkey: JUPITER_FORK_ACCOUNTS.dovesPrice, isSigner: false, isWritable: false },
    // 6: custody_pyth_price - Pyth oracle price
    { pubkey: JUPITER_FORK_ACCOUNTS.pythPrice, isSigner: false, isWritable: false },
    // 7: custody_token_account (writable) - SPL token account for USDC
    { pubkey: JUPITER_FORK_ACCOUNTS.usdcTokenCustody, isSigner: false, isWritable: true },
    // 8: lp_token_mint (writable) - LP token mint (JLP)
    { pubkey: JUPITER_FORK_ACCOUNTS.lpTokenMint, isSigner: false, isWritable: true },
    // 9: event_authority - Event authority for emissions
    { pubkey: JUPITER_FORK_ACCOUNTS.eventAuthority, isSigner: false, isWritable: false },
    // 10: Jupiter program (again for CPI check)
    { pubkey: JUPITER_FORK_ACCOUNTS.program, isSigner: false, isWritable: false },
    // 11: vault_lp token account (writable) - provided separately to this function
    { pubkey: vaultLpTokenAccount, isSigner: false, isWritable: true },
    // 12: SPL Token program
    { pubkey: TOKEN_PROGRAM, isSigner: false, isWritable: false },
  ];
}

/**
 * Remaining accounts for Jupiter withdraw CPI (remove_liquidity2)
 * Same structure as deposit
 */
export function buildJupiterWithdrawRemainingAccounts(
  transferAuthority: PublicKey,
  vaultLpTokenAccount: PublicKey
): AccountMeta[] {
  // Same as deposit per protocol.rs implementation
  return buildJupiterDepositRemainingAccounts(transferAuthority, vaultLpTokenAccount);
}

/**
 * WARNING: Jupiter account addresses need verification!
 * 
 * Current addresses are placeholders. To get the correct addresses:
 * 1. Check Jupiter's on-chain program state
 * 2. Query the perpetuals state to find pool account
 * 3. Query pool to find custody, LP token mint, etc.
 * 4. Find the correct oracle price feeds for USDC
 * 
 * TODO: Update these with actual mainnet addresses once verified
 */
