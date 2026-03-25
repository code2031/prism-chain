"use client";

import { useState } from "react";
import type { MarketData } from "./market-card";
import {
  calculateInterestRate,
  calculateUtilization,
  calculateDepositRate,
  calculateAPY,
  formatTokenAmount,
} from "../lib/interest-math";

interface SupplyModalProps {
  market: MarketData;
  onClose: () => void;
}

export default function SupplyModal({ market, onClose }: SupplyModalProps) {
  const [amount, setAmount] = useState("");
  const walletBalance = 12_500; // Mock wallet balance

  const utilBps = calculateUtilization(market.totalDeposits, market.totalBorrows);
  const borrowRateBps = calculateInterestRate(utilBps);
  const depositRateBps = calculateDepositRate(borrowRateBps, utilBps);
  const currentAPY = calculateAPY(depositRateBps);

  const parsedAmount = parseFloat(amount) || 0;

  // Projected APY after this deposit
  const newDeposits = market.totalDeposits + parsedAmount;
  const newUtilBps = calculateUtilization(newDeposits, market.totalBorrows);
  const newBorrowRate = calculateInterestRate(newUtilBps);
  const newDepositRate = calculateDepositRate(newBorrowRate, newUtilBps);
  const projectedAPY = calculateAPY(newDepositRate);

  const handleMax = () => {
    setAmount(walletBalance.toString());
  };

  const handleSupply = () => {
    if (parsedAmount > 0 && parsedAmount <= walletBalance) {
      alert(
        `Supplying ${formatTokenAmount(parsedAmount)} ${market.symbol} at ${currentAPY}% APY`,
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
              Supply {market.symbol}
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

        {/* Current APY */}
        <div className="mb-5 rounded-xl bg-green-500/10 p-4 text-center">
          <p className="text-sm text-gray-400">Current Supply APY</p>
          <p className="text-3xl font-bold text-green-400">{currentAPY}%</p>
        </div>

        {/* Amount input */}
        <div className="mb-4">
          <div className="mb-2 flex items-center justify-between">
            <label className="text-sm font-medium text-gray-400">Amount</label>
            <span className="text-sm text-gray-500">
              Balance: {formatTokenAmount(walletBalance)} {market.symbol}
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
              onClick={handleMax}
              className="rounded-lg bg-purple-600/20 px-3 py-1 text-xs font-bold text-purple-400 transition hover:bg-purple-600/30"
            >
              MAX
            </button>
          </div>
        </div>

        {/* Info rows */}
        <div className="mb-6 space-y-3 rounded-xl bg-gray-800/50 p-4">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Supply APY</span>
            <span className="text-green-400">{currentAPY}%</span>
          </div>
          {parsedAmount > 0 && (
            <div className="flex items-center justify-between text-sm">
              <span className="text-gray-400">Projected APY</span>
              <span className="text-green-300">{projectedAPY}%</span>
            </div>
          )}
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Utilization</span>
            <span className="text-gray-300">{(utilBps / 100).toFixed(1)}%</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Total Market Supply</span>
            <span className="text-gray-300">
              ${formatTokenAmount(market.totalDeposits)}
            </span>
          </div>
        </div>

        {/* Supply button */}
        <button
          onClick={handleSupply}
          disabled={parsedAmount <= 0 || parsedAmount > walletBalance}
          className="w-full rounded-xl bg-green-600 py-3.5 text-base font-bold text-white transition hover:bg-green-500 disabled:cursor-not-allowed disabled:opacity-40"
        >
          {parsedAmount <= 0
            ? "Enter an amount"
            : parsedAmount > walletBalance
              ? "Insufficient balance"
              : `Supply ${formatTokenAmount(parsedAmount)} ${market.symbol}`}
        </button>
      </div>
    </div>
  );
}
