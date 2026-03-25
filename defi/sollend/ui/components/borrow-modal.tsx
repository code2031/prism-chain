"use client";

import { useState } from "react";
import type { MarketData } from "./market-card";
import {
  calculateInterestRate,
  calculateUtilization,
  calculateHealthFactor,
  calculateAPY,
  formatTokenAmount,
  LTV_RATIO_BPS,
} from "../lib/interest-math";

interface BorrowModalProps {
  market: MarketData;
  onClose: () => void;
}

export default function BorrowModal({ market, onClose }: BorrowModalProps) {
  const [amount, setAmount] = useState("");

  // Mock user position
  const userDeposited = 10_000;
  const userBorrowed = 2_000;

  const utilBps = calculateUtilization(market.totalDeposits, market.totalBorrows);
  const borrowRateBps = calculateInterestRate(utilBps);
  const borrowAPY = calculateAPY(borrowRateBps);

  const parsedAmount = parseFloat(amount) || 0;
  const maxBorrow = (userDeposited * LTV_RATIO_BPS) / 10_000 - userBorrowed;

  // Current health factor
  const currentHF = calculateHealthFactor(userDeposited, userBorrowed);

  // Projected health factor
  const newBorrowed = userBorrowed + parsedAmount;
  const projectedHF = calculateHealthFactor(userDeposited, newBorrowed);

  // Projected utilization / APY
  const newTotalBorrows = market.totalBorrows + parsedAmount;
  const newUtilBps = calculateUtilization(market.totalDeposits, newTotalBorrows);
  const newBorrowRate = calculateInterestRate(newUtilBps);
  const projectedBorrowAPY = calculateAPY(newBorrowRate);

  const formatHF = (hf: number) => {
    if (!isFinite(hf)) return "---";
    return hf.toFixed(2);
  };

  const hfColor = (hf: number) => {
    if (!isFinite(hf)) return "text-green-400";
    if (hf >= 1.5) return "text-green-400";
    if (hf >= 1.2) return "text-yellow-400";
    return "text-red-400";
  };

  const handleBorrow = () => {
    if (parsedAmount > 0 && parsedAmount <= maxBorrow) {
      alert(
        `Borrowing ${formatTokenAmount(parsedAmount)} ${market.symbol} at ${borrowAPY}% APY`,
      );
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-2xl border border-gray-800 bg-gray-900 p-6 shadow-2xl">
        {/* Header */}
        <div className="mb-6 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">{market.icon}</span>
            <h2 className="text-xl font-bold text-white">
              Borrow {market.symbol}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="rounded-lg p-1 text-gray-400 transition hover:bg-gray-800 hover:text-white"
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
              <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
            </svg>
          </button>
        </div>

        {/* Borrow APY */}
        <div className="mb-5 rounded-xl bg-red-500/10 p-4 text-center">
          <p className="text-sm text-gray-400">Borrow APY</p>
          <p className="text-3xl font-bold text-red-400">{borrowAPY}%</p>
        </div>

        {/* Amount input */}
        <div className="mb-4">
          <div className="mb-2 flex items-center justify-between">
            <label className="text-sm font-medium text-gray-400">
              Borrow Amount
            </label>
            <span className="text-sm text-gray-500">
              Max: {formatTokenAmount(Math.max(maxBorrow, 0))} {market.symbol}
            </span>
          </div>
          <div className="flex items-center gap-2 rounded-xl border border-gray-700 bg-gray-800 px-4 py-3 focus-within:border-purple-500">
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="flex-1 bg-transparent text-xl font-semibold text-white outline-none placeholder:text-gray-600"
            />
            <button
              onClick={() => setAmount(Math.max(maxBorrow, 0).toString())}
              className="rounded-lg bg-purple-600/20 px-3 py-1 text-xs font-bold text-purple-400 transition hover:bg-purple-600/30"
            >
              MAX
            </button>
          </div>
        </div>

        {/* Info rows */}
        <div className="mb-6 space-y-3 rounded-xl bg-gray-800/50 p-4">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Borrow APY</span>
            <span className="text-red-400">{borrowAPY}%</span>
          </div>
          {parsedAmount > 0 && (
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-400">Projected Borrow APY</span>
              <span className="text-red-300">{projectedBorrowAPY}%</span>
            </div>
          )}
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Collateral (Your Deposits)</span>
            <span className="text-gray-300">
              ${formatTokenAmount(userDeposited)}
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Max LTV</span>
            <span className="text-gray-300">80%</span>
          </div>
          <div className="border-t border-gray-700 pt-3">
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-400">Health Factor</span>
              <div className="flex items-center gap-2">
                <span className={hfColor(currentHF)}>{formatHF(currentHF)}</span>
                {parsedAmount > 0 && (
                  <>
                    <span className="text-gray-600">-&gt;</span>
                    <span className={hfColor(projectedHF)}>
                      {formatHF(projectedHF)}
                    </span>
                  </>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Borrow button */}
        <button
          onClick={handleBorrow}
          disabled={parsedAmount <= 0 || parsedAmount > maxBorrow}
          className="w-full rounded-xl bg-purple-600 py-3.5 text-base font-bold text-white transition hover:bg-purple-500 disabled:cursor-not-allowed disabled:opacity-40"
        >
          {parsedAmount <= 0
            ? "Enter an amount"
            : parsedAmount > maxBorrow
              ? "Exceeds borrow limit"
              : `Borrow ${formatTokenAmount(parsedAmount)} ${market.symbol}`}
        </button>
      </div>
    </div>
  );
}
