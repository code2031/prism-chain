"use client";

import { useState } from "react";
import SwapForm from "../components/swap-form";
import PoolCard from "../components/pool-card";
import AddLiquidityModal from "../components/add-liquidity-modal";

// ---------------------------------------------------------------------------
// Mock pool data
// ---------------------------------------------------------------------------

interface Pool {
  tokenA: string;
  tokenB: string;
  tvl: number;
  volume24h: number;
  apr: number;
  yourShare: number;
  reserveA: number;
  reserveB: number;
  lpSupply: number;
  balanceA: number;
  balanceB: number;
}

const POOLS: Pool[] = [
  {
    tokenA: "SOL",
    tokenB: "USDC",
    tvl: 15_240_000,
    volume24h: 4_820_000,
    apr: 28.4,
    yourShare: 0.12,
    reserveA: 50_000,
    reserveB: 7_500_000,
    lpSupply: 612_000,
    balanceA: 24.56,
    balanceB: 1842.33,
  },
  {
    tokenA: "SOL",
    tokenB: "USDT",
    tvl: 11_800_000,
    volume24h: 3_150_000,
    apr: 22.1,
    yourShare: 0,
    reserveA: 48_000,
    reserveB: 7_200_000,
    lpSupply: 588_000,
    balanceA: 24.56,
    balanceB: 500.0,
  },
  {
    tokenA: "USDC",
    tokenB: "USDT",
    tvl: 20_000_000,
    volume24h: 8_900_000,
    apr: 8.2,
    yourShare: 0.03,
    reserveA: 10_000_000,
    reserveB: 10_000_000,
    lpSupply: 10_000_000,
    balanceA: 1842.33,
    balanceB: 500.0,
  },
  {
    tokenA: "SOL",
    tokenB: "RAY",
    tvl: 4_200_000,
    volume24h: 1_340_000,
    apr: 42.7,
    yourShare: 0,
    reserveA: 20_000,
    reserveB: 1_200_000,
    lpSupply: 155_000,
    balanceA: 24.56,
    balanceB: 312.8,
  },
  {
    tokenA: "USDC",
    tokenB: "RAY",
    tvl: 2_100_000,
    volume24h: 680_000,
    apr: 31.5,
    yourShare: 0,
    reserveA: 5_000_000,
    reserveB: 300_000,
    lpSupply: 1_224_000,
    balanceA: 1842.33,
    balanceB: 312.8,
  },
  {
    tokenA: "SOL",
    tokenB: "BONK",
    tvl: 1_800_000,
    volume24h: 920_000,
    apr: 68.3,
    yourShare: 0,
    reserveA: 10_000,
    reserveB: 500_000_000_000,
    lpSupply: 70_710,
    balanceA: 24.56,
    balanceB: 5_000_000,
  },
];

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

type Tab = "swap" | "pools";

// ---------------------------------------------------------------------------
// Page Component
// ---------------------------------------------------------------------------

export default function Home() {
  const [activeTab, setActiveTab] = useState<Tab>("swap");
  const [liquidityModal, setLiquidityModal] = useState<Pool | null>(null);

  return (
    <div className="bg-grid-pattern flex flex-1 flex-col">
      {/* Hero / stats banner */}
      <section className="mx-auto w-full max-w-6xl px-4 pt-10 sm:px-6">
        <div className="mb-8 text-center">
          <h1 className="mb-2 text-3xl font-extrabold tracking-tight sm:text-4xl">
            Trade on <span className="gradient-text">SolClone</span>
          </h1>
          <p className="text-base text-gray-400">
            Lightning-fast token swaps with 0.25% fees. Provide liquidity and earn.
          </p>
        </div>

        {/* Quick stats */}
        <div className="mx-auto mb-10 grid max-w-2xl grid-cols-3 gap-4">
          <div className="rounded-xl border border-white/[0.04] bg-white/[0.02] px-4 py-3 text-center">
            <div className="text-xs font-medium uppercase tracking-wider text-gray-500">
              Total TVL
            </div>
            <div className="mt-1 text-lg font-bold text-white">$55.14M</div>
          </div>
          <div className="rounded-xl border border-white/[0.04] bg-white/[0.02] px-4 py-3 text-center">
            <div className="text-xs font-medium uppercase tracking-wider text-gray-500">
              24h Volume
            </div>
            <div className="mt-1 text-lg font-bold text-white">$19.81M</div>
          </div>
          <div className="rounded-xl border border-white/[0.04] bg-white/[0.02] px-4 py-3 text-center">
            <div className="text-xs font-medium uppercase tracking-wider text-gray-500">
              Total Pools
            </div>
            <div className="mt-1 text-lg font-bold text-white">{POOLS.length}</div>
          </div>
        </div>
      </section>

      {/* Tab navigation */}
      <div className="mx-auto w-full max-w-6xl px-4 sm:px-6">
        <div className="mb-8 flex items-center justify-center gap-1 rounded-xl border border-white/[0.04] bg-white/[0.02] p-1">
          <button
            type="button"
            onClick={() => setActiveTab("swap")}
            className={`rounded-lg px-6 py-2.5 text-sm font-semibold transition ${
              activeTab === "swap"
                ? "bg-gradient-to-r from-purple-600/80 to-green-500/80 text-white shadow-md"
                : "text-gray-400 hover:text-white"
            }`}
          >
            Swap
          </button>
          <button
            type="button"
            onClick={() => setActiveTab("pools")}
            className={`rounded-lg px-6 py-2.5 text-sm font-semibold transition ${
              activeTab === "pools"
                ? "bg-gradient-to-r from-purple-600/80 to-green-500/80 text-white shadow-md"
                : "text-gray-400 hover:text-white"
            }`}
          >
            Pools
          </button>
        </div>
      </div>

      {/* Tab content */}
      <section className="mx-auto w-full max-w-6xl flex-1 px-4 pb-16 sm:px-6">
        {activeTab === "swap" && (
          <div className="animate-fade-in-up">
            <SwapForm />
          </div>
        )}

        {activeTab === "pools" && (
          <div className="animate-fade-in-up">
            <div className="mb-6 flex items-center justify-between">
              <h2 className="text-lg font-bold">Liquidity Pools</h2>
              <div className="flex items-center gap-2 text-sm text-gray-400">
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Earn fees by providing liquidity
              </div>
            </div>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {POOLS.map((pool) => (
                <PoolCard
                  key={`${pool.tokenA}-${pool.tokenB}`}
                  tokenA={pool.tokenA}
                  tokenB={pool.tokenB}
                  tvl={pool.tvl}
                  volume24h={pool.volume24h}
                  apr={pool.apr}
                  yourShare={pool.yourShare}
                  onAddLiquidity={() => setLiquidityModal(pool)}
                />
              ))}
            </div>
          </div>
        )}
      </section>

      {/* Add Liquidity Modal */}
      {liquidityModal && (
        <AddLiquidityModal
          tokenA={liquidityModal.tokenA}
          tokenB={liquidityModal.tokenB}
          reserveA={liquidityModal.reserveA}
          reserveB={liquidityModal.reserveB}
          lpSupply={liquidityModal.lpSupply}
          balanceA={liquidityModal.balanceA}
          balanceB={liquidityModal.balanceB}
          onClose={() => setLiquidityModal(null)}
        />
      )}
    </div>
  );
}
