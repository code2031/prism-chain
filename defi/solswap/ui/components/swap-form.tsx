"use client";

import { useState, useMemo, useCallback } from "react";
import {
  calculateSwapOutputNumber,
  calculatePriceImpact,
  formatPriceImpact,
  getSpotPrice,
  calculateMinimumReceived,
} from "../lib/amm-math";

// ---------------------------------------------------------------------------
// Mock token list
// ---------------------------------------------------------------------------

interface Token {
  symbol: string;
  name: string;
  logo: string;
  balance: number;
  decimals: number;
}

const TOKENS: Token[] = [
  { symbol: "SOL",  name: "Solana",    logo: "/tokens/sol.svg",  balance: 24.5612,  decimals: 9 },
  { symbol: "USDC", name: "USD Coin",  logo: "/tokens/usdc.svg", balance: 1842.33,  decimals: 6 },
  { symbol: "USDT", name: "Tether",    logo: "/tokens/usdt.svg", balance: 500.0,    decimals: 6 },
  { symbol: "RAY",  name: "Raydium",   logo: "/tokens/ray.svg",  balance: 312.8,    decimals: 6 },
  { symbol: "SRM",  name: "Serum",     logo: "/tokens/srm.svg",  balance: 0,        decimals: 6 },
  { symbol: "BONK", name: "Bonk",      logo: "/tokens/bonk.svg", balance: 5_000_000, decimals: 5 },
];

// Mock reserve data keyed by "SYMBOLA/SYMBOLB".
const MOCK_RESERVES: Record<string, { reserveA: number; reserveB: number }> = {
  "SOL/USDC":  { reserveA: 50_000,       reserveB: 7_500_000 },
  "SOL/USDT":  { reserveA: 48_000,       reserveB: 7_200_000 },
  "SOL/RAY":   { reserveA: 20_000,       reserveB: 1_200_000 },
  "USDC/USDT": { reserveA: 10_000_000,   reserveB: 10_000_000 },
  "USDC/RAY":  { reserveA: 5_000_000,    reserveB: 300_000 },
  "SOL/BONK":  { reserveA: 10_000,       reserveB: 500_000_000_000 },
};

function getReserves(
  symbolA: string,
  symbolB: string,
): { reserveIn: number; reserveOut: number } | null {
  const keyDirect = `${symbolA}/${symbolB}`;
  const keyReverse = `${symbolB}/${symbolA}`;
  if (MOCK_RESERVES[keyDirect]) {
    return {
      reserveIn: MOCK_RESERVES[keyDirect].reserveA,
      reserveOut: MOCK_RESERVES[keyDirect].reserveB,
    };
  }
  if (MOCK_RESERVES[keyReverse]) {
    return {
      reserveIn: MOCK_RESERVES[keyReverse].reserveB,
      reserveOut: MOCK_RESERVES[keyReverse].reserveA,
    };
  }
  return null;
}

// ---------------------------------------------------------------------------
// Slippage presets
// ---------------------------------------------------------------------------

const SLIPPAGE_PRESETS = [50, 100, 200]; // basis points (0.5%, 1%, 2%)

// ---------------------------------------------------------------------------
// Token Selector Dropdown
// ---------------------------------------------------------------------------

function TokenSelector({
  selected,
  onSelect,
  exclude,
}: {
  selected: Token;
  onSelect: (t: Token) => void;
  exclude?: string;
}) {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex items-center gap-2 rounded-xl bg-white/5 px-3 py-2 text-sm font-semibold transition hover:bg-white/10"
      >
        <span className="flex h-6 w-6 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-indigo-600 text-[10px] font-bold text-white">
          {selected.symbol[0]}
        </span>
        <span>{selected.symbol}</span>
        <svg className="h-4 w-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {open && (
        <div className="absolute left-0 top-full z-50 mt-2 w-56 rounded-xl border border-white/10 bg-[#1a1a2e] p-2 shadow-2xl">
          {TOKENS.filter((t) => t.symbol !== exclude).map((t) => (
            <button
              key={t.symbol}
              type="button"
              onClick={() => { onSelect(t); setOpen(false); }}
              className="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm transition hover:bg-white/10"
            >
              <span className="flex h-7 w-7 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-indigo-600 text-[10px] font-bold text-white">
                {t.symbol[0]}
              </span>
              <div className="flex-1">
                <div className="font-semibold">{t.symbol}</div>
                <div className="text-xs text-gray-400">{t.name}</div>
              </div>
              <span className="text-xs text-gray-500">{t.balance.toLocaleString()}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// SwapForm
// ---------------------------------------------------------------------------

export default function SwapForm() {
  const [tokenFrom, setTokenFrom] = useState<Token>(TOKENS[0]); // SOL
  const [tokenTo, setTokenTo] = useState<Token>(TOKENS[1]);     // USDC
  const [amountIn, setAmountIn] = useState<string>("");
  const [slippageBps, setSlippageBps] = useState<number>(50);
  const [customSlippage, setCustomSlippage] = useState<string>("");
  const [showSettings, setShowSettings] = useState(false);
  const [swapping, setSwapping] = useState(false);

  // Compute output -----------------------------------------------------------

  const reserves = useMemo(
    () => getReserves(tokenFrom.symbol, tokenTo.symbol),
    [tokenFrom.symbol, tokenTo.symbol],
  );

  const inputAmount = parseFloat(amountIn) || 0;

  const swapOutput = useMemo(() => {
    if (!reserves || inputAmount <= 0) return 0;
    return calculateSwapOutputNumber(
      inputAmount,
      reserves.reserveIn,
      reserves.reserveOut,
    );
  }, [inputAmount, reserves]);

  const priceImpactBps = useMemo(() => {
    if (!reserves || inputAmount <= 0 || swapOutput <= 0) return 0;
    return calculatePriceImpact(
      inputAmount,
      swapOutput,
      reserves.reserveIn,
      reserves.reserveOut,
    );
  }, [inputAmount, swapOutput, reserves]);

  const spotPrice = useMemo(() => {
    if (!reserves) return 0;
    return getSpotPrice(reserves.reserveIn, reserves.reserveOut);
  }, [reserves]);

  const minReceived = useMemo(() => {
    return calculateMinimumReceived(swapOutput, slippageBps);
  }, [swapOutput, slippageBps]);

  // Handlers -----------------------------------------------------------------

  const handleReverse = useCallback(() => {
    setTokenFrom(tokenTo);
    setTokenTo(tokenFrom);
    setAmountIn("");
  }, [tokenFrom, tokenTo]);

  const handleMax = useCallback(() => {
    setAmountIn(tokenFrom.balance.toString());
  }, [tokenFrom.balance]);

  const handleSwap = useCallback(async () => {
    setSwapping(true);
    // Simulate transaction delay
    await new Promise((r) => setTimeout(r, 1500));
    setSwapping(false);
    setAmountIn("");
  }, []);

  const activeSlippage = customSlippage
    ? Math.round(parseFloat(customSlippage) * 100)
    : slippageBps;

  const impactColor =
    priceImpactBps > 500
      ? "text-red-400"
      : priceImpactBps > 100
        ? "text-yellow-400"
        : "text-green-400";

  const canSwap =
    inputAmount > 0 &&
    inputAmount <= tokenFrom.balance &&
    reserves !== null &&
    swapOutput > 0;

  // ---------------------------------------------------------------------------

  return (
    <div className="mx-auto w-full max-w-[460px]">
      {/* Card container */}
      <div className="relative rounded-2xl border border-white/[0.06] bg-[#12121a] p-5 shadow-2xl shadow-purple-900/10">
        {/* Header row */}
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-bold">Swap</h2>
          <button
            type="button"
            onClick={() => setShowSettings(!showSettings)}
            className="rounded-lg p-2 text-gray-400 transition hover:bg-white/5 hover:text-white"
            aria-label="Settings"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.573-1.066z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>

        {/* Slippage settings panel */}
        {showSettings && (
          <div className="mb-4 rounded-xl border border-white/[0.06] bg-white/[0.03] p-4">
            <div className="mb-2 text-sm font-medium text-gray-300">Slippage Tolerance</div>
            <div className="flex gap-2">
              {SLIPPAGE_PRESETS.map((preset) => (
                <button
                  key={preset}
                  type="button"
                  onClick={() => { setSlippageBps(preset); setCustomSlippage(""); }}
                  className={`rounded-lg px-3 py-1.5 text-sm font-medium transition ${
                    slippageBps === preset && !customSlippage
                      ? "bg-purple-600 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10 hover:text-white"
                  }`}
                >
                  {preset / 100}%
                </button>
              ))}
              <div className="relative flex-1">
                <input
                  type="number"
                  step="0.1"
                  min="0.01"
                  max="50"
                  value={customSlippage}
                  onChange={(e) => {
                    setCustomSlippage(e.target.value);
                    const val = parseFloat(e.target.value);
                    if (!isNaN(val) && val > 0) setSlippageBps(Math.round(val * 100));
                  }}
                  placeholder="Custom"
                  className="w-full rounded-lg bg-white/5 px-3 py-1.5 text-sm text-white placeholder-gray-500 outline-none ring-1 ring-transparent focus:ring-purple-500"
                />
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-sm text-gray-500">
                  %
                </span>
              </div>
            </div>
          </div>
        )}

        {/* --- FROM token input --- */}
        <div className="rounded-xl border border-white/[0.04] bg-white/[0.02] p-4">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-xs font-medium uppercase tracking-wider text-gray-500">From</span>
            <span className="text-xs text-gray-500">
              Balance: {tokenFrom.balance.toLocaleString()}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <TokenSelector
              selected={tokenFrom}
              onSelect={setTokenFrom}
              exclude={tokenTo.symbol}
            />
            <input
              type="number"
              inputMode="decimal"
              value={amountIn}
              onChange={(e) => setAmountIn(e.target.value)}
              placeholder="0.0"
              className="min-w-0 flex-1 bg-transparent text-right text-2xl font-semibold text-white outline-none placeholder-gray-600"
            />
          </div>
          <div className="mt-2 flex items-center justify-between">
            <button
              type="button"
              onClick={handleMax}
              className="rounded-md bg-purple-600/20 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-purple-400 transition hover:bg-purple-600/30"
            >
              MAX
            </button>
            {inputAmount > 0 && spotPrice > 0 && (
              <span className="text-xs text-gray-500">
                ~${(inputAmount * (spotPrice > 1 ? spotPrice : 1 / spotPrice > 100 ? 150 : spotPrice)).toLocaleString(undefined, { maximumFractionDigits: 2 })}
              </span>
            )}
          </div>
        </div>

        {/* Swap direction arrow */}
        <div className="relative z-10 -my-3 flex justify-center">
          <button
            type="button"
            onClick={handleReverse}
            className="flex h-10 w-10 items-center justify-center rounded-xl border-4 border-[#12121a] bg-[#1e1e30] text-gray-400 transition hover:rotate-180 hover:text-white"
            aria-label="Reverse swap direction"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
            </svg>
          </button>
        </div>

        {/* --- TO token output --- */}
        <div className="rounded-xl border border-white/[0.04] bg-white/[0.02] p-4">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-xs font-medium uppercase tracking-wider text-gray-500">To</span>
            <span className="text-xs text-gray-500">
              Balance: {tokenTo.balance.toLocaleString()}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <TokenSelector
              selected={tokenTo}
              onSelect={setTokenTo}
              exclude={tokenFrom.symbol}
            />
            <div className="min-w-0 flex-1 text-right text-2xl font-semibold text-white">
              {swapOutput > 0
                ? swapOutput.toLocaleString(undefined, {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 6,
                  })
                : <span className="text-gray-600">0.0</span>
              }
            </div>
          </div>
        </div>

        {/* --- Swap details --- */}
        {inputAmount > 0 && swapOutput > 0 && (
          <div className="mt-4 space-y-2 rounded-xl border border-white/[0.04] bg-white/[0.02] px-4 py-3 text-sm">
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Rate</span>
              <span className="text-gray-200">
                1 {tokenFrom.symbol} = {spotPrice.toLocaleString(undefined, { maximumFractionDigits: 6 })} {tokenTo.symbol}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Price Impact</span>
              <span className={impactColor}>{formatPriceImpact(priceImpactBps)}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Minimum Received</span>
              <span className="text-gray-200">
                {minReceived.toLocaleString(undefined, { maximumFractionDigits: 6 })} {tokenTo.symbol}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Slippage Tolerance</span>
              <span className="text-gray-200">{activeSlippage / 100}%</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Fee (0.25%)</span>
              <span className="text-gray-200">
                {(inputAmount * 0.0025).toLocaleString(undefined, { maximumFractionDigits: 6 })} {tokenFrom.symbol}
              </span>
            </div>
          </div>
        )}

        {/* No route warning */}
        {reserves === null && tokenFrom.symbol !== tokenTo.symbol && (
          <div className="mt-4 rounded-xl border border-yellow-500/20 bg-yellow-500/5 px-4 py-3 text-center text-sm text-yellow-400">
            No liquidity pool for {tokenFrom.symbol}/{tokenTo.symbol}
          </div>
        )}

        {/* Swap button */}
        <button
          type="button"
          disabled={!canSwap || swapping}
          onClick={handleSwap}
          className={`mt-5 flex w-full items-center justify-center rounded-xl py-4 text-base font-bold transition ${
            canSwap && !swapping
              ? "bg-gradient-to-r from-purple-600 to-green-500 text-white shadow-lg shadow-purple-600/25 hover:shadow-purple-600/40 hover:brightness-110"
              : "cursor-not-allowed bg-white/5 text-gray-600"
          }`}
        >
          {swapping ? (
            <div className="flex items-center gap-2">
              <svg className="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              Swapping...
            </div>
          ) : !inputAmount || inputAmount <= 0 ? (
            "Enter an amount"
          ) : inputAmount > tokenFrom.balance ? (
            `Insufficient ${tokenFrom.symbol} balance`
          ) : reserves === null ? (
            "No route found"
          ) : (
            "Swap"
          )}
        </button>
      </div>
    </div>
  );
}
