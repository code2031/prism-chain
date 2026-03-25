"use client";

import {
  calculateInterestRate,
  calculateUtilization,
  calculateDepositRate,
  calculateAPY,
  formatTokenAmount,
} from "../lib/interest-math";

export interface MarketData {
  symbol: string;
  name: string;
  icon: string;
  totalDeposits: number;
  totalBorrows: number;
}

interface MarketCardProps {
  market: MarketData;
  onSupply: (market: MarketData) => void;
  onBorrow: (market: MarketData) => void;
}

export default function MarketCard({
  market,
  onSupply,
  onBorrow,
}: MarketCardProps) {
  const utilBps = calculateUtilization(market.totalDeposits, market.totalBorrows);
  const utilPct = utilBps / 100;
  const borrowRateBps = calculateInterestRate(utilBps);
  const depositRateBps = calculateDepositRate(borrowRateBps, utilBps);
  const supplyAPY = calculateAPY(depositRateBps);
  const borrowAPY = calculateAPY(borrowRateBps);

  return (
    <div className="rounded-2xl border border-gray-800 bg-gray-900/60 p-6 backdrop-blur transition hover:border-purple-600/50 hover:shadow-lg hover:shadow-purple-900/20">
      {/* Header */}
      <div className="mb-5 flex items-center gap-3">
        <span className="text-3xl">{market.icon}</span>
        <div>
          <h3 className="text-lg font-bold text-white">{market.symbol}</h3>
          <p className="text-sm text-gray-400">{market.name}</p>
        </div>
      </div>

      {/* Stats grid */}
      <div className="mb-5 grid grid-cols-2 gap-4">
        <div>
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Total Supply
          </p>
          <p className="mt-1 text-lg font-semibold text-white">
            ${formatTokenAmount(market.totalDeposits)}
          </p>
        </div>
        <div>
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Total Borrowed
          </p>
          <p className="mt-1 text-lg font-semibold text-white">
            ${formatTokenAmount(market.totalBorrows)}
          </p>
        </div>
        <div>
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Supply APY
          </p>
          <p className="mt-1 text-lg font-semibold text-green-400">
            {supplyAPY}%
          </p>
        </div>
        <div>
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Borrow APY
          </p>
          <p className="mt-1 text-lg font-semibold text-red-400">
            {borrowAPY}%
          </p>
        </div>
      </div>

      {/* Utilization bar */}
      <div className="mb-5">
        <div className="mb-1 flex items-center justify-between">
          <span className="text-xs font-medium text-gray-500">Utilization</span>
          <span className="text-xs font-semibold text-gray-300">
            {utilPct.toFixed(1)}%
          </span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-gray-800">
          <div
            className="h-full rounded-full transition-all duration-500"
            style={{
              width: `${Math.min(utilPct, 100)}%`,
              background:
                utilPct > 90
                  ? "linear-gradient(90deg, #ef4444, #dc2626)"
                  : utilPct > 70
                    ? "linear-gradient(90deg, #a855f7, #ec4899)"
                    : "linear-gradient(90deg, #22c55e, #a855f7)",
            }}
          />
        </div>
      </div>

      {/* Action buttons */}
      <div className="flex gap-3">
        <button
          onClick={() => onSupply(market)}
          className="flex-1 rounded-xl bg-green-600/20 px-4 py-2.5 text-sm font-semibold text-green-400 transition hover:bg-green-600/30"
        >
          Supply
        </button>
        <button
          onClick={() => onBorrow(market)}
          className="flex-1 rounded-xl bg-purple-600/20 px-4 py-2.5 text-sm font-semibold text-purple-400 transition hover:bg-purple-600/30"
        >
          Borrow
        </button>
      </div>
    </div>
  );
}
