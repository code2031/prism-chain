"use client";

interface PoolCardProps {
  tokenA: string;
  tokenB: string;
  tvl: number;
  volume24h: number;
  apr: number;
  yourShare: number;
  onAddLiquidity: () => void;
}

function formatUsd(value: number): string {
  if (value >= 1_000_000) return `$${(value / 1_000_000).toFixed(2)}M`;
  if (value >= 1_000) return `$${(value / 1_000).toFixed(1)}K`;
  return `$${value.toFixed(2)}`;
}

export default function PoolCard({
  tokenA,
  tokenB,
  tvl,
  volume24h,
  apr,
  yourShare,
  onAddLiquidity,
}: PoolCardProps) {
  return (
    <div className="group rounded-2xl border border-white/[0.06] bg-[#12121a] p-5 transition hover:border-purple-500/30 hover:shadow-lg hover:shadow-purple-900/10">
      {/* Token pair header */}
      <div className="mb-4 flex items-center gap-3">
        <div className="relative flex">
          <span className="flex h-9 w-9 items-center justify-center rounded-full bg-gradient-to-br from-purple-500 to-indigo-600 text-xs font-bold text-white ring-2 ring-[#12121a]">
            {tokenA[0]}
          </span>
          <span className="-ml-2 flex h-9 w-9 items-center justify-center rounded-full bg-gradient-to-br from-green-400 to-emerald-600 text-xs font-bold text-white ring-2 ring-[#12121a]">
            {tokenB[0]}
          </span>
        </div>
        <div>
          <h3 className="text-base font-bold">
            {tokenA}/{tokenB}
          </h3>
          <span className="text-xs text-gray-500">0.25% fee tier</span>
        </div>
        {apr > 20 && (
          <span className="ml-auto rounded-md bg-green-500/10 px-2 py-0.5 text-xs font-semibold text-green-400">
            HOT
          </span>
        )}
      </div>

      {/* Stats grid */}
      <div className="mb-4 grid grid-cols-2 gap-3">
        <div className="rounded-lg bg-white/[0.03] px-3 py-2">
          <div className="text-[11px] font-medium uppercase tracking-wider text-gray-500">TVL</div>
          <div className="mt-0.5 text-sm font-semibold text-white">{formatUsd(tvl)}</div>
        </div>
        <div className="rounded-lg bg-white/[0.03] px-3 py-2">
          <div className="text-[11px] font-medium uppercase tracking-wider text-gray-500">Vol 24h</div>
          <div className="mt-0.5 text-sm font-semibold text-white">{formatUsd(volume24h)}</div>
        </div>
        <div className="rounded-lg bg-white/[0.03] px-3 py-2">
          <div className="text-[11px] font-medium uppercase tracking-wider text-gray-500">APR</div>
          <div className="mt-0.5 text-sm font-semibold text-green-400">{apr.toFixed(1)}%</div>
        </div>
        <div className="rounded-lg bg-white/[0.03] px-3 py-2">
          <div className="text-[11px] font-medium uppercase tracking-wider text-gray-500">Your Share</div>
          <div className="mt-0.5 text-sm font-semibold text-white">
            {yourShare > 0 ? `${yourShare.toFixed(2)}%` : "--"}
          </div>
        </div>
      </div>

      {/* Add liquidity button */}
      <button
        type="button"
        onClick={onAddLiquidity}
        className="flex w-full items-center justify-center gap-2 rounded-xl border border-purple-500/30 bg-purple-600/10 py-2.5 text-sm font-semibold text-purple-400 transition hover:bg-purple-600/20 hover:text-purple-300"
      >
        <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
        </svg>
        Add Liquidity
      </button>
    </div>
  );
}
