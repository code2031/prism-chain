"use client";

import React from "react";

interface StatCardProps {
  label: string;
  value: string | number;
  trend?: "up" | "down" | "neutral";
  icon?: React.ReactNode;
}

export default function StatCard({ label, value, trend, icon }: StatCardProps) {
  const trendArrow =
    trend === "up"
      ? "\u2191"
      : trend === "down"
        ? "\u2193"
        : null;

  const trendColor =
    trend === "up"
      ? "text-[#14F195]"
      : trend === "down"
        ? "text-red-400"
        : "";

  return (
    <div className="rounded-xl border border-white/10 bg-[#1A1440] p-5 flex flex-col gap-2 shadow-lg">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-400 uppercase tracking-wide">
          {label}
        </span>
        {icon && <span className="text-[#9945FF]">{icon}</span>}
      </div>
      <div className="flex items-end gap-2">
        <span className="text-3xl font-bold text-white tabular-nums">
          {value}
        </span>
        {trendArrow && (
          <span className={`text-lg font-semibold ${trendColor} pb-0.5`}>
            {trendArrow}
          </span>
        )}
      </div>
    </div>
  );
}
