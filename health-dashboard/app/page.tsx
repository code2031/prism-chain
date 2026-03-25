"use client";

import React from "react";
import { useClusterStats } from "@/lib/rpc-poller";
import StatCard from "@/components/stat-card";
import TpsChart from "@/components/tps-chart";
import ValidatorTable from "@/components/validator-table";
import EpochProgress from "@/components/epoch-progress";

export default function HomePage() {
  const { data, error, isLoading, isValidating } = useClusterStats();

  return (
    <main className="min-h-screen bg-[#0F0B2E] text-white">
      {/* Header */}
      <header className="border-b border-white/10 px-6 py-4">
        <div className="mx-auto flex max-w-7xl items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-[#9945FF] to-[#14F195]" />
            <h1 className="text-xl font-bold tracking-tight">
              SolClone Network
            </h1>
          </div>
          <div className="flex items-center gap-3">
            {/* Auto-refresh indicator */}
            <div className="flex items-center gap-2 text-xs text-gray-400">
              <span
                className={`h-2 w-2 rounded-full ${
                  isValidating
                    ? "bg-[#9945FF] animate-pulse"
                    : "bg-[#14F195]"
                }`}
              />
              {isValidating ? "Refreshing..." : "Live"}
            </div>
          </div>
        </div>
      </header>

      {/* Content */}
      <div className="mx-auto max-w-7xl px-6 py-8 space-y-6">
        {/* Error state */}
        {error && (
          <div className="rounded-xl border border-red-500/30 bg-red-500/10 p-4 text-red-300 text-sm">
            Failed to fetch cluster data: {error.message}. Retrying...
          </div>
        )}

        {/* Loading state */}
        {isLoading && !data && (
          <div className="flex items-center justify-center py-32">
            <div className="flex flex-col items-center gap-4">
              <div className="h-10 w-10 animate-spin rounded-full border-2 border-[#9945FF] border-t-transparent" />
              <p className="text-gray-400 text-sm">
                Connecting to SolClone network...
              </p>
            </div>
          </div>
        )}

        {data && (
          <>
            {/* Stat cards row */}
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <StatCard
                label="Slot Height"
                value={data.slot.toLocaleString()}
                trend="up"
              />
              <StatCard
                label="TPS"
                value={data.tps.toLocaleString()}
                trend={data.tps > 0 ? "up" : "neutral"}
              />
              <StatCard
                label="Validators"
                value={data.validatorCount.toLocaleString()}
                trend="neutral"
              />
              <StatCard
                label="Epoch"
                value={data.epoch.toLocaleString()}
                trend="up"
              />
            </div>

            {/* TPS Chart */}
            {data.tpsSamples.length > 0 && (
              <TpsChart samples={data.tpsSamples} />
            )}

            {/* Epoch progress */}
            <EpochProgress
              epoch={data.epoch}
              slotIndex={data.slotIndex}
              slotsInEpoch={data.slotsInEpoch}
              progress={data.epochProgress}
            />

            {/* Validator table */}
            {data.validators.length > 0 && (
              <ValidatorTable validators={data.validators} />
            )}
          </>
        )}
      </div>

      {/* Footer */}
      <footer className="border-t border-white/10 px-6 py-4 mt-8">
        <div className="mx-auto max-w-7xl flex items-center justify-between text-xs text-gray-500">
          <span>SolClone Network Health Dashboard</span>
          <span>Polling every 2s via Solana JSON-RPC</span>
        </div>
      </footer>
    </main>
  );
}
