"use client";

import { calculateHealthFactor, formatTokenAmount } from "../lib/interest-math";

interface PositionItem {
  symbol: string;
  icon: string;
  amount: number;
  value: number;
  apy: number;
}

export interface UserPositions {
  supplied: PositionItem[];
  borrowed: PositionItem[];
}

interface PositionPanelProps {
  positions: UserPositions;
}

export default function PositionPanel({ positions }: PositionPanelProps) {
  const totalSupplied = positions.supplied.reduce((s, p) => s + p.value, 0);
  const totalBorrowed = positions.borrowed.reduce((s, p) => s + p.value, 0);
  const healthFactor = calculateHealthFactor(totalSupplied, totalBorrowed);

  // Weighted net APY
  const supplyYield = positions.supplied.reduce(
    (s, p) => s + p.value * (p.apy / 100),
    0,
  );
  const borrowCost = positions.borrowed.reduce(
    (s, p) => s + p.value * (p.apy / 100),
    0,
  );
  const netAPY =
    totalSupplied > 0
      ? ((supplyYield - borrowCost) / totalSupplied) * 100
      : 0;

  // Health factor display
  const hfDisplay = isFinite(healthFactor) ? healthFactor.toFixed(2) : "---";
  const hfColor = !isFinite(healthFactor)
    ? "text-green-400"
    : healthFactor >= 1.5
      ? "text-green-400"
      : healthFactor >= 1.2
        ? "text-yellow-400"
        : "text-red-400";
  const hfBg = !isFinite(healthFactor)
    ? "bg-green-500"
    : healthFactor >= 1.5
      ? "bg-green-500"
      : healthFactor >= 1.2
        ? "bg-yellow-500"
        : "bg-red-500";
  // Gauge percentage: map HF 0..2 to 0..100
  const hfPct = isFinite(healthFactor)
    ? Math.min((healthFactor / 2) * 100, 100)
    : 100;

  const hasPositions =
    positions.supplied.length > 0 || positions.borrowed.length > 0;

  if (!hasPositions) {
    return (
      <div className="rounded-2xl border border-gray-800 bg-gray-900/60 p-8 text-center backdrop-blur">
        <p className="text-lg text-gray-500">
          No open positions. Supply assets to get started.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Summary row */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-4">
        <div className="rounded-xl border border-gray-800 bg-gray-900/60 p-4 backdrop-blur">
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Total Supplied
          </p>
          <p className="mt-1 text-xl font-bold text-white">
            ${formatTokenAmount(totalSupplied)}
          </p>
        </div>
        <div className="rounded-xl border border-gray-800 bg-gray-900/60 p-4 backdrop-blur">
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Total Borrowed
          </p>
          <p className="mt-1 text-xl font-bold text-white">
            ${formatTokenAmount(totalBorrowed)}
          </p>
        </div>
        <div className="rounded-xl border border-gray-800 bg-gray-900/60 p-4 backdrop-blur">
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Net APY
          </p>
          <p
            className={`mt-1 text-xl font-bold ${netAPY >= 0 ? "text-green-400" : "text-red-400"}`}
          >
            {netAPY >= 0 ? "+" : ""}
            {netAPY.toFixed(2)}%
          </p>
        </div>
        <div className="rounded-xl border border-gray-800 bg-gray-900/60 p-4 backdrop-blur">
          <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
            Health Factor
          </p>
          <p className={`mt-1 text-xl font-bold ${hfColor}`}>{hfDisplay}</p>
          {/* Gauge bar */}
          <div className="mt-2 h-1.5 w-full overflow-hidden rounded-full bg-gray-800">
            <div
              className={`h-full rounded-full transition-all duration-500 ${hfBg}`}
              style={{ width: `${hfPct}%` }}
            />
          </div>
        </div>
      </div>

      {/* Supplied assets */}
      {positions.supplied.length > 0 && (
        <div className="rounded-2xl border border-gray-800 bg-gray-900/60 backdrop-blur">
          <div className="border-b border-gray-800 px-6 py-4">
            <h3 className="text-base font-bold text-white">Your Supplies</h3>
          </div>
          <div className="divide-y divide-gray-800">
            {positions.supplied.map((pos) => (
              <div
                key={pos.symbol}
                className="flex items-center justify-between px-6 py-4"
              >
                <div className="flex items-center gap-3">
                  <span className="text-2xl">{pos.icon}</span>
                  <div>
                    <p className="font-semibold text-white">{pos.symbol}</p>
                    <p className="text-sm text-gray-400">
                      {formatTokenAmount(pos.amount)} tokens
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-semibold text-white">
                    ${formatTokenAmount(pos.value)}
                  </p>
                  <p className="text-sm text-green-400">+{pos.apy.toFixed(2)}% APY</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Borrowed assets */}
      {positions.borrowed.length > 0 && (
        <div className="rounded-2xl border border-gray-800 bg-gray-900/60 backdrop-blur">
          <div className="border-b border-gray-800 px-6 py-4">
            <h3 className="text-base font-bold text-white">Your Borrows</h3>
          </div>
          <div className="divide-y divide-gray-800">
            {positions.borrowed.map((pos) => (
              <div
                key={pos.symbol}
                className="flex items-center justify-between px-6 py-4"
              >
                <div className="flex items-center gap-3">
                  <span className="text-2xl">{pos.icon}</span>
                  <div>
                    <p className="font-semibold text-white">{pos.symbol}</p>
                    <p className="text-sm text-gray-400">
                      {formatTokenAmount(pos.amount)} tokens
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-semibold text-white">
                    ${formatTokenAmount(pos.value)}
                  </p>
                  <p className="text-sm text-red-400">-{pos.apy.toFixed(2)}% APY</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
