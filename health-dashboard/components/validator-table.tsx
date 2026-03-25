"use client";

import React from "react";
import type { ValidatorInfo } from "@/lib/rpc-poller";

interface ValidatorTableProps {
  validators: ValidatorInfo[];
}

function truncateAddress(addr: string): string {
  if (addr.length <= 12) return addr;
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

function formatStake(lamports: number): string {
  const sol = lamports / 1_000_000_000;
  if (sol >= 1_000_000) return `${(sol / 1_000_000).toFixed(2)}M`;
  if (sol >= 1_000) return `${(sol / 1_000).toFixed(2)}K`;
  return sol.toFixed(2);
}

export default function ValidatorTable({ validators }: ValidatorTableProps) {
  // Show top 50 by stake
  const display = validators.slice(0, 50);

  return (
    <div className="rounded-xl border border-white/10 bg-[#1A1440] p-5 shadow-lg overflow-hidden">
      <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wide mb-4">
        Validators ({validators.length} total)
      </h2>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-white/10 text-gray-400 text-xs uppercase">
              <th className="text-left py-2 pr-4">#</th>
              <th className="text-left py-2 pr-4">Identity</th>
              <th className="text-right py-2 pr-4">Stake (SOL)</th>
              <th className="text-right py-2 pr-4">Commission</th>
              <th className="text-right py-2 pr-4">Last Vote</th>
              <th className="text-center py-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {display.map((v, i) => (
              <tr
                key={v.identity}
                className="border-b border-white/5 hover:bg-white/5 transition-colors"
              >
                <td className="py-2.5 pr-4 text-gray-500 tabular-nums">{i + 1}</td>
                <td className="py-2.5 pr-4 font-mono text-white">
                  {truncateAddress(v.identity)}
                </td>
                <td className="py-2.5 pr-4 text-right text-white tabular-nums">
                  {formatStake(v.stake)}
                </td>
                <td className="py-2.5 pr-4 text-right text-white tabular-nums">
                  {v.commission}%
                </td>
                <td className="py-2.5 pr-4 text-right text-gray-300 tabular-nums">
                  {v.lastVote.toLocaleString()}
                </td>
                <td className="py-2.5 text-center">
                  {v.delinquent ? (
                    <span className="inline-flex items-center gap-1 rounded-full bg-red-500/20 px-2.5 py-0.5 text-xs font-medium text-red-400">
                      <span className="h-1.5 w-1.5 rounded-full bg-red-400" />
                      Delinquent
                    </span>
                  ) : (
                    <span className="inline-flex items-center gap-1 rounded-full bg-[#14F195]/20 px-2.5 py-0.5 text-xs font-medium text-[#14F195]">
                      <span className="h-1.5 w-1.5 rounded-full bg-[#14F195]" />
                      Active
                    </span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
