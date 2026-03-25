// =============================================================================
// SolSwap AMM Math — Client-side pure functions
// =============================================================================
//
// These functions mirror the on-chain constant-product AMM logic so the UI can
// preview swap outputs, price impact, and LP token calculations without
// submitting a transaction.
// =============================================================================

/** Fee defaults matching the on-chain program. */
export const DEFAULT_FEE_NUMERATOR = 25n;
export const DEFAULT_FEE_DENOMINATOR = 10_000n;
export const DEFAULT_PROTOCOL_FEE_NUMERATOR = 5n;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Integer square root (Newton / Babylonian method) for bigint values. */
export function sqrt(value: bigint): bigint {
  if (value < 0n) throw new Error("sqrt of negative");
  if (value === 0n) return 0n;
  let x = value;
  let y = (x + 1n) / 2n;
  while (y < x) {
    x = y;
    y = (x + value / x) / 2n;
  }
  return x;
}

// ---------------------------------------------------------------------------
// Swap output
// ---------------------------------------------------------------------------

export interface SwapResult {
  /** Tokens the user receives. */
  amountOut: bigint;
  /** Total fee charged (in input-token units). */
  totalFee: bigint;
  /** Subset of totalFee routed to the protocol treasury. */
  protocolFee: bigint;
}

/**
 * Constant-product swap with fee:
 *
 *   effectiveIn = amountIn * (feeDenominator - feeNumerator)
 *   amountOut   = effectiveIn * reserveOut
 *                 / (reserveIn * feeDenominator + effectiveIn)
 */
export function calculateSwapOutput(
  amountIn: bigint,
  reserveIn: bigint,
  reserveOut: bigint,
  feeNumerator: bigint = DEFAULT_FEE_NUMERATOR,
  feeDenominator: bigint = DEFAULT_FEE_DENOMINATOR,
  protocolFeeNumerator: bigint = DEFAULT_PROTOCOL_FEE_NUMERATOR,
): SwapResult {
  if (amountIn <= 0n) return { amountOut: 0n, totalFee: 0n, protocolFee: 0n };
  if (reserveIn <= 0n || reserveOut <= 0n)
    return { amountOut: 0n, totalFee: 0n, protocolFee: 0n };

  const effectiveIn = amountIn * (feeDenominator - feeNumerator);
  const numerator = effectiveIn * reserveOut;
  const denominator = reserveIn * feeDenominator + effectiveIn;
  const amountOut = numerator / denominator;

  const totalFee = (amountIn * feeNumerator) / feeDenominator;
  const protocolFee = (amountIn * protocolFeeNumerator) / feeDenominator;

  return { amountOut, totalFee, protocolFee };
}

/**
 * Convenience wrapper that accepts plain numbers (token amounts with decimals
 * already resolved) and returns plain numbers.
 */
export function calculateSwapOutputNumber(
  amountIn: number,
  reserveIn: number,
  reserveOut: number,
  feeNumerator: number = Number(DEFAULT_FEE_NUMERATOR),
  feeDenominator: number = Number(DEFAULT_FEE_DENOMINATOR),
): number {
  if (amountIn <= 0 || reserveIn <= 0 || reserveOut <= 0) return 0;
  const effectiveIn = amountIn * (feeDenominator - feeNumerator);
  const numerator = effectiveIn * reserveOut;
  const denominator = reserveIn * feeDenominator + effectiveIn;
  return numerator / denominator;
}

// ---------------------------------------------------------------------------
// Price impact
// ---------------------------------------------------------------------------

/**
 * Price impact in basis points.
 *
 *   spotPrice  = reserveOut / reserveIn
 *   execPrice  = amountOut  / amountIn
 *   impactBps  = (1 - execPrice / spotPrice) * 10 000
 */
export function calculatePriceImpact(
  amountIn: number,
  amountOut: number,
  reserveIn: number,
  reserveOut: number,
): number {
  if (amountIn <= 0 || reserveIn <= 0 || reserveOut <= 0) return 0;

  const spotPrice = reserveOut / reserveIn;
  const execPrice = amountOut / amountIn;

  const impact = (1 - execPrice / spotPrice) * 10_000;
  return Math.max(0, Math.round(impact));
}

/**
 * Return the price impact as a human-friendly percentage string.
 */
export function formatPriceImpact(bps: number): string {
  const pct = bps / 100;
  if (pct < 0.01) return "< 0.01%";
  return `${pct.toFixed(2)}%`;
}

// ---------------------------------------------------------------------------
// LP token calculation
// ---------------------------------------------------------------------------

/**
 * Calculate LP tokens minted for a deposit.
 *
 *   First deposit:  sqrt(amountA * amountB)
 *   Subsequent:     min(amountA * lpSupply / reserveA,
 *                       amountB * lpSupply / reserveB)
 */
export function calculateLpTokens(
  amountA: bigint,
  amountB: bigint,
  reserveA: bigint,
  reserveB: bigint,
  lpSupply: bigint,
): bigint {
  if (amountA <= 0n || amountB <= 0n) return 0n;

  if (lpSupply === 0n) {
    return sqrt(amountA * amountB);
  }

  const lpA = (amountA * lpSupply) / reserveA;
  const lpB = (amountB * lpSupply) / reserveB;
  return lpA < lpB ? lpA : lpB;
}

/**
 * Number-based convenience wrapper for LP calculation.
 */
export function calculateLpTokensNumber(
  amountA: number,
  amountB: number,
  reserveA: number,
  reserveB: number,
  lpSupply: number,
): number {
  if (amountA <= 0 || amountB <= 0) return 0;

  if (lpSupply === 0) {
    return Math.sqrt(amountA * amountB);
  }

  const lpA = (amountA * lpSupply) / reserveA;
  const lpB = (amountB * lpSupply) / reserveB;
  return Math.min(lpA, lpB);
}

// ---------------------------------------------------------------------------
// Withdrawal calculation
// ---------------------------------------------------------------------------

export interface WithdrawalResult {
  amountA: number;
  amountB: number;
}

/**
 * Calculate the amount of each underlying token received when burning LP
 * tokens.
 */
export function calculateWithdrawal(
  lpAmount: number,
  lpSupply: number,
  reserveA: number,
  reserveB: number,
): WithdrawalResult {
  if (lpAmount <= 0 || lpSupply <= 0) return { amountA: 0, amountB: 0 };
  return {
    amountA: (lpAmount * reserveA) / lpSupply,
    amountB: (lpAmount * reserveB) / lpSupply,
  };
}

// ---------------------------------------------------------------------------
// Share of pool
// ---------------------------------------------------------------------------

/**
 * Calculate the provider's share of the pool after depositing, expressed as a
 * percentage (0-100).
 */
export function calculatePoolShare(
  lpTokensToMint: number,
  currentLpSupply: number,
): number {
  const newSupply = currentLpSupply + lpTokensToMint;
  if (newSupply <= 0) return 0;
  return (lpTokensToMint / newSupply) * 100;
}

// ---------------------------------------------------------------------------
// Exchange rate
// ---------------------------------------------------------------------------

/**
 * Spot exchange rate: how many of tokenOut you get per 1 tokenIn (before
 * fees), based on current reserves.
 */
export function getSpotPrice(reserveIn: number, reserveOut: number): number {
  if (reserveIn <= 0) return 0;
  return reserveOut / reserveIn;
}

/**
 * Minimum received given a slippage tolerance (in basis points).
 */
export function calculateMinimumReceived(
  expectedOutput: number,
  slippageBps: number,
): number {
  return expectedOutput * (1 - slippageBps / 10_000);
}
