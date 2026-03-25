"use client";

import { useState, useMemo, useCallback, useEffect, useRef } from "react";
import {
  calculateLpTokensNumber,
  calculatePoolShare,
  getSpotPrice,
} from "../lib/amm-math";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface AddLiquidityModalProps {
  tokenA: string;
  tokenB: string;
  reserveA: number;
  reserveB: number;
  lpSupply: number;
  balanceA: number;
  balanceB: number;
  onClose: () => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export default function AddLiquidityModal({
  tokenA,
  tokenB,
  reserveA,
  reserveB,
  lpSupply,
  balanceA,
  balanceB,
  onClose,
}: AddLiquidityModalProps) {
  const [amountA, setAmountA] = useState<string>("");
  const [amountB, setAmountB] = useState<string>("");
  const [submitting, setSubmitting] = useState(false);
  const backdropRef = useRef<HTMLDivElement>(null);

  const parsedA = parseFloat(amountA) || 0;
  const parsedB = parseFloat(amountB) || 0;

  // Auto-calculate the paired amount based on the pool ratio.
  const autoCalcB = useCallback(
    (aVal: string) => {
      const a = parseFloat(aVal);
      if (!a || reserveA <= 0) {
        setAmountB("");
        return;
      }
      const ratio = reserveB / reserveA;
      setAmountB((a * ratio).toFixed(6));
    },
    [reserveA, reserveB],
  );

  const autoCalcA = useCallback(
    (bVal: string) => {
      const b = parseFloat(bVal);
      if (!b || reserveB <= 0) {
        setAmountA("");
        return;
      }
      const ratio = reserveA / reserveB;
      setAmountA((b * ratio).toFixed(6));
    },
    [reserveA, reserveB],
  );

  // LP tokens to receive
  const lpTokens = useMemo(() => {
    if (parsedA <= 0 || parsedB <= 0) return 0;
    return calculateLpTokensNumber(parsedA, parsedB, reserveA, reserveB, lpSupply);
  }, [parsedA, parsedB, reserveA, reserveB, lpSupply]);

  // Share of pool
  const sharePercent = useMemo(() => {
    return calculatePoolShare(lpTokens, lpSupply);
  }, [lpTokens, lpSupply]);

  // Spot price
  const price = useMemo(() => getSpotPrice(reserveA, reserveB), [reserveA, reserveB]);

  const canSubmit =
    parsedA > 0 &&
    parsedB > 0 &&
    parsedA <= balanceA &&
    parsedB <= balanceB &&
    !submitting;

  const handleSubmit = useCallback(async () => {
    setSubmitting(true);
    await new Promise((r) => setTimeout(r, 1500));
    setSubmitting(false);
    onClose();
  }, [onClose]);

  // Close on backdrop click
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (e.target === backdropRef.current) onClose();
    },
    [onClose],
  );

  // Close on Escape
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  return (
    <div
      ref={backdropRef}
      onClick={handleBackdropClick}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    >
      <div className="relative w-full max-w-md rounded-2xl border border-white/[0.08] bg-[#12121a] p-6 shadow-2xl">
        {/* Close button */}
        <button
          type="button"
          onClick={onClose}
          className="absolute right-4 top-4 rounded-lg p-1.5 text-gray-400 transition hover:bg-white/5 hover:text-white"
          aria-label="Close"
        >
          <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        <h2 className="mb-1 text-lg font-bold text-white">Add Liquidity</h2>
        <p className="mb-5 text-sm text-gray-400">
          Deposit tokens to the {tokenA}/{tokenB} pool and earn fees.
        </p>

        {/* Token A input */}
        <div className="mb-3 rounded-xl border border-white/[0.04] bg-white/[0.02] p-4">
          <div className="mb-2 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-indigo-600 text-[10px] font-bold text-white">
                {tokenA[0]}
              </span>
              <span className="text-sm font-semibold">{tokenA}</span>
            </div>
            <span className="text-xs text-gray-500">
              Balance: {balanceA.toLocaleString()}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="number"
              inputMode="decimal"
              value={amountA}
              onChange={(e) => {
                setAmountA(e.target.value);
                autoCalcB(e.target.value);
              }}
              placeholder="0.0"
              className="min-w-0 flex-1 bg-transparent text-xl font-semibold text-white outline-none placeholder-gray-600"
            />
            <button
              type="button"
              onClick={() => {
                setAmountA(balanceA.toString());
                autoCalcB(balanceA.toString());
              }}
              className="rounded-md bg-purple-600/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-purple-400 transition hover:bg-purple-600/30"
            >
              MAX
            </button>
          </div>
        </div>

        {/* Plus icon */}
        <div className="relative z-10 -my-1.5 flex justify-center">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg border-4 border-[#12121a] bg-[#1e1e30] text-gray-400">
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </div>
        </div>

        {/* Token B input */}
        <div className="mb-4 rounded-xl border border-white/[0.04] bg-white/[0.02] p-4">
          <div className="mb-2 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-br from-green-400 to-emerald-600 text-[10px] font-bold text-white">
                {tokenB[0]}
              </span>
              <span className="text-sm font-semibold">{tokenB}</span>
            </div>
            <span className="text-xs text-gray-500">
              Balance: {balanceB.toLocaleString()}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="number"
              inputMode="decimal"
              value={amountB}
              onChange={(e) => {
                setAmountB(e.target.value);
                autoCalcA(e.target.value);
              }}
              placeholder="0.0"
              className="min-w-0 flex-1 bg-transparent text-xl font-semibold text-white outline-none placeholder-gray-600"
            />
            <button
              type="button"
              onClick={() => {
                setAmountB(balanceB.toString());
                autoCalcA(balanceB.toString());
              }}
              className="rounded-md bg-green-600/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-green-400 transition hover:bg-green-600/30"
            >
              MAX
            </button>
          </div>
        </div>

        {/* Details */}
        <div className="mb-5 space-y-2 rounded-xl border border-white/[0.04] bg-white/[0.02] px-4 py-3 text-sm">
          <div className="flex items-center justify-between">
            <span className="text-gray-400">Pool Price</span>
            <span className="text-gray-200">
              1 {tokenA} = {price.toLocaleString(undefined, { maximumFractionDigits: 4 })} {tokenB}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-400">LP Tokens Received</span>
            <span className="text-gray-200">
              {lpTokens > 0
                ? lpTokens.toLocaleString(undefined, { maximumFractionDigits: 6 })
                : "--"}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-400">Share of Pool</span>
            <span className="text-purple-400 font-medium">
              {sharePercent > 0 ? `${sharePercent.toFixed(4)}%` : "--"}
            </span>
          </div>
        </div>

        {/* Submit button */}
        <button
          type="button"
          disabled={!canSubmit}
          onClick={handleSubmit}
          className={`flex w-full items-center justify-center rounded-xl py-4 text-base font-bold transition ${
            canSubmit
              ? "bg-gradient-to-r from-purple-600 to-green-500 text-white shadow-lg shadow-purple-600/25 hover:shadow-purple-600/40 hover:brightness-110"
              : "cursor-not-allowed bg-white/5 text-gray-600"
          }`}
        >
          {submitting ? (
            <div className="flex items-center gap-2">
              <svg className="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              Adding Liquidity...
            </div>
          ) : parsedA <= 0 || parsedB <= 0 ? (
            "Enter amounts"
          ) : parsedA > balanceA ? (
            `Insufficient ${tokenA}`
          ) : parsedB > balanceB ? (
            `Insufficient ${tokenB}`
          ) : (
            "Add Liquidity"
          )}
        </button>
      </div>
    </div>
  );
}
