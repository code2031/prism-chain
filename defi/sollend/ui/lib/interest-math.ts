// ---------------------------------------------------------------------------
// SolLend – Client-side interest rate mathematics
// ---------------------------------------------------------------------------
// All rates are expressed in basis points (bps): 10 000 bps = 100 %.
// The kinked (two-slope) curve mirrors the on-chain program exactly.
// ---------------------------------------------------------------------------

const BPS = 10_000;

// Model parameters (bps per year)
const BASE_RATE_BPS = 200; // 2 %
const SLOPE1_BPS = 400; // 4 % (0 – 80 % utilisation)
const SLOPE2_BPS = 7_500; // 75 % (80 – 100 % utilisation)
const OPTIMAL_UTILIZATION_BPS = 8_000; // 80 %

// Risk parameters
export const LTV_RATIO_BPS = 8_000; // 80 %
export const LIQUIDATION_THRESHOLD_BPS = 8_500; // 85 %
export const LIQUIDATION_BONUS_BPS = 500; // 5 %

/**
 * Calculate the annualised borrow rate (bps) for a given utilisation (bps).
 *
 * Kinked curve:
 *   util <= optimal  =>  base + slope1 * (util / optimal)
 *   util >  optimal  =>  base + slope1 + slope2 * ((util - optimal) / (1 - optimal))
 */
export function calculateInterestRate(utilizationBps: number): number {
  if (utilizationBps <= 0) return BASE_RATE_BPS;

  if (utilizationBps <= OPTIMAL_UTILIZATION_BPS) {
    const variable = (SLOPE1_BPS * utilizationBps) / OPTIMAL_UTILIZATION_BPS;
    return BASE_RATE_BPS + variable;
  }

  const excess = utilizationBps - OPTIMAL_UTILIZATION_BPS;
  const remaining = BPS - OPTIMAL_UTILIZATION_BPS; // 2 000
  const steep = (SLOPE2_BPS * excess) / remaining;
  return BASE_RATE_BPS + SLOPE1_BPS + steep;
}

/**
 * Utilisation = totalBorrows / totalDeposits, returned in bps.
 */
export function calculateUtilization(
  totalDeposits: number,
  totalBorrows: number,
): number {
  if (totalDeposits === 0) return 0;
  return Math.min((totalBorrows * BPS) / totalDeposits, BPS);
}

/**
 * Health factor = (deposited * liquidationThreshold) / (borrowed * BPS).
 * Returns a float where 1.0 means exactly at liquidation boundary.
 */
export function calculateHealthFactor(
  deposited: number,
  borrowed: number,
): number {
  if (borrowed === 0) return Infinity;
  return (deposited * LIQUIDATION_THRESHOLD_BPS) / (borrowed * BPS);
}

/**
 * Convert a bps rate to a human-readable APY percentage string.
 * e.g. 600 bps => "6.00"
 */
export function calculateAPY(rateBps: number): string {
  return (rateBps / 100).toFixed(2);
}

/**
 * Deposit APY = borrowRate * utilization / BPS  (protocol takes no spread).
 */
export function calculateDepositRate(
  borrowRateBps: number,
  utilizationBps: number,
): number {
  return (borrowRateBps * utilizationBps) / BPS;
}

/**
 * Format a large token amount with commas and fixed decimals.
 */
export function formatTokenAmount(
  amount: number,
  decimals: number = 2,
): string {
  return amount.toLocaleString("en-US", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}
