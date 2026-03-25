"use client";

import React from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

interface TpsChartProps {
  samples: number[];
}

export default function TpsChart({ samples }: TpsChartProps) {
  const data = samples.map((tps, i) => ({
    index: i,
    tps,
  }));

  return (
    <div className="rounded-xl border border-white/10 bg-[#1A1440] p-5 shadow-lg">
      <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wide mb-4">
        TPS &mdash; Last {samples.length} Samples
      </h2>
      <div className="h-64 w-full">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data} margin={{ top: 5, right: 10, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="tpsGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#9945FF" stopOpacity={0.5} />
                <stop offset="95%" stopColor="#9945FF" stopOpacity={0.0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.06)" />
            <XAxis
              dataKey="index"
              tick={{ fill: "#6b7280", fontSize: 11 }}
              axisLine={{ stroke: "rgba(255,255,255,0.1)" }}
              tickLine={false}
            />
            <YAxis
              tick={{ fill: "#6b7280", fontSize: 11 }}
              axisLine={{ stroke: "rgba(255,255,255,0.1)" }}
              tickLine={false}
              width={50}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: "#1A1440",
                border: "1px solid rgba(255,255,255,0.15)",
                borderRadius: "8px",
                color: "#fff",
                fontSize: 13,
              }}
              labelFormatter={(v) => `Sample ${v}`}
              formatter={(value) => [`${Number(value).toLocaleString()} tx/s`, "TPS"]}
            />
            <Area
              type="monotone"
              dataKey="tps"
              stroke="#9945FF"
              strokeWidth={2}
              fill="url(#tpsGradient)"
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
