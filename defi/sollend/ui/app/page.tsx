"use client";

import { useState } from "react";
import MarketCard, { type MarketData } from "../components/market-card";
import SupplyModal from "../components/supply-modal";
import BorrowModal from "../components/borrow-modal";
import PositionPanel, { type UserPositions } from "../components/position-panel";
import {
  calculateInterestRate,
  calculateUtilization,
  calculateDepositRate,
} from "../lib/interest-math";

// ---------------------------------------------------------------------------
// Mock market data
// ---------------------------------------------------------------------------
const MARKETS: MarketData[] = [
  {
    symbol: "SOL",
    name: "Solana",
    icon: "\u25c8",
    totalDeposits: 12_450_000,
    totalBorrows: 7_820_000,
  },
  {
    symbol: "USDC",
    name: "USD Coin",
    icon: "\u25cb",
    totalDeposits: 28_300_000,
    totalBorrows: 19_100_000,
  },
  {
    symbol: "ETH",
    name: "Ethereum",
    icon: "\u25ca",
    totalDeposits: 8_750_000,
    totalBorrows: 3_200_000,
  },
  {
    symbol: "BTC",
    name: "Bitcoin",
    icon: "\u0e3f",
    totalDeposits: 15_600_000,
    totalBorrows: 5_400_000,
  },
  {
    symbol: "BONK",
    name: "Bonk",
    icon: "\u2726",
    totalDeposits: 2_100_000,
    totalBorrows: 1_680_000,
  },
  {
    symbol: "JUP",
    name: "Jupiter",
    icon: "\u2643",
    totalDeposits: 4_300_000,
    totalBorrows: 1_200_000,
  },
];

// ---------------------------------------------------------------------------
// Mock user positions (derived from markets for realism)
// ---------------------------------------------------------------------------
function buildMockPositions(): UserPositions {
  const solUtil = calculateUtilization(12_450_000, 7_820_000);
  const solBorrowRate = calculateInterestRate(solUtil);
  const solDepositRate = calculateDepositRate(solBorrowRate, solUtil);

  const usdcUtil = calculateUtilization(28_300_000, 19_100_000);
  const usdcBorrowRate = calculateInterestRate(usdcUtil);
  const usdcDepositRate = calculateDepositRate(usdcBorrowRate, usdcUtil);

  const ethUtil = calculateUtilization(8_750_000, 3_200_000);
  const ethBorrowRate = calculateInterestRate(ethUtil);
  const ethDepositRate = calculateDepositRate(ethBorrowRate, ethUtil);

  return {
    supplied: [
      {
        symbol: "SOL",
        icon: "\u25c8",
        amount: 420,
        value: 58_800,
        apy: solDepositRate / 100,
      },
      {
        symbol: "ETH",
        icon: "\u25ca",
        amount: 3.2,
        value: 10_240,
        apy: ethDepositRate / 100,
      },
    ],
    borrowed: [
      {
        symbol: "USDC",
        icon: "\u25cb",
        amount: 12_000,
        value: 12_000,
        apy: usdcBorrowRate / 100,
      },
    ],
  };
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------
export default function Home() {
  const [supplyMarket, setSupplyMarket] = useState<MarketData | null>(null);
  const [borrowMarket, setBorrowMarket] = useState<MarketData | null>(null);

  const positions = buildMockPositions();

  const totalTVL = MARKETS.reduce((s, m) => s + m.totalDeposits, 0);

  return (
    <div className="flex min-h-screen flex-col">
      {/* ---- Header ---- */}
      <header className="sticky top-0 z-40 border-b border-gray-800/60 bg-gray-950/80 backdrop-blur-lg">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-purple-600 text-lg font-black text-white">
              S
            </div>
            <span className="text-xl font-bold tracking-tight text-white">
              Sol<span className="text-purple-400">Lend</span>
            </span>
          </div>
          <div className="hidden items-center gap-6 text-sm font-medium text-gray-400 sm:flex">
            <span className="text-white">Markets</span>
            <span className="cursor-pointer transition hover:text-white">
              Dashboard
            </span>
            <span className="cursor-pointer transition hover:text-white">
              Governance
            </span>
            <span className="cursor-pointer transition hover:text-white">
              Docs
            </span>
          </div>
          <button className="rounded-xl bg-purple-600 px-5 py-2 text-sm font-semibold text-white transition hover:bg-purple-500">
            Connect Wallet
          </button>
        </div>
      </header>

      {/* ---- Hero stats ---- */}
      <section className="border-b border-gray-800/40">
        <div className="mx-auto max-w-7xl px-6 py-10">
          <h1 className="mb-2 text-3xl font-bold text-white sm:text-4xl">
            Lend &amp; Borrow on{" "}
            <span className="bg-gradient-to-r from-purple-400 to-green-400 bg-clip-text text-transparent">
              SolClone
            </span>
          </h1>
          <p className="mb-8 max-w-xl text-gray-400">
            Earn interest on your deposits and borrow against your collateral
            with the SolLend decentralized lending protocol.
          </p>
          <div className="flex flex-wrap gap-8">
            <div>
              <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
                Total Value Locked
              </p>
              <p className="mt-1 text-2xl font-bold text-white">
                ${(totalTVL / 1_000_000).toFixed(1)}M
              </p>
            </div>
            <div>
              <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
                Markets
              </p>
              <p className="mt-1 text-2xl font-bold text-white">
                {MARKETS.length}
              </p>
            </div>
            <div>
              <p className="text-xs font-medium uppercase tracking-wider text-gray-500">
                Protocol
              </p>
              <p className="mt-1 text-2xl font-bold text-purple-400">
                SolLend v1
              </p>
            </div>
          </div>
        </div>
      </section>

      <main className="mx-auto w-full max-w-7xl flex-1 px-6 py-10">
        {/* ---- Markets ---- */}
        <section className="mb-14">
          <h2 className="mb-6 text-2xl font-bold text-white">Markets</h2>
          <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {MARKETS.map((m) => (
              <MarketCard
                key={m.symbol}
                market={m}
                onSupply={setSupplyMarket}
                onBorrow={setBorrowMarket}
              />
            ))}
          </div>
        </section>

        {/* ---- Your Positions ---- */}
        <section>
          <h2 className="mb-6 text-2xl font-bold text-white">
            Your Positions
          </h2>
          <PositionPanel positions={positions} />
        </section>
      </main>

      {/* ---- Footer ---- */}
      <footer className="border-t border-gray-800/40 py-6 text-center text-sm text-gray-600">
        SolLend Protocol &mdash; Built on SolClone
      </footer>

      {/* ---- Modals ---- */}
      {supplyMarket && (
        <SupplyModal
          market={supplyMarket}
          onClose={() => setSupplyMarket(null)}
        />
      )}
      {borrowMarket && (
        <BorrowModal
          market={borrowMarket}
          onClose={() => setBorrowMarket(null)}
        />
      )}
    </div>
  );
}
