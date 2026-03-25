"use client";

import React from "react";

interface EpochProgressProps {
  epoch: number;
  slotIndex: number;
  slotsInEpoch: number;
  progress: number; // 0-100
}

export default function EpochProgress({
  epoch,
  slotIndex,
  slotsInEpoch,
  progress,
}: EpochProgressProps) {
  return (
    <div className="rounded-xl border border-white/10 bg-[#1A1440] p-5 shadow-lg">
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wide">
          Epoch {epoch} Progress
        </h2>
        <span className="text-sm font-semibold text-[#14F195]">
          {progress.toFixed(1)}%
        </span>
      </div>

      {/* Progress bar */}
      <div className="h-3 w-full rounded-full bg-white/10 overflow-hidden">
        <div
          className="h-full rounded-full transition-all duration-500 ease-out"
          style={{
            width: `${Math.min(progress, 100)}%`,
            background: "linear-gradient(90deg, #9945FF 0%, #14F195 100%)",
          }}
        />
      </div>

      {/* Slot counts */}
      <div className="flex items-center justify-between mt-3 text-xs text-gray-400">
        <span>
          Slot <span className="text-white font-medium">{slotIndex.toLocaleString()}</span>{" "}
          of {slotsInEpoch.toLocaleString()}
        </span>
        <span>
          Remaining:{" "}
          <span className="text-white font-medium">
            {(slotsInEpoch - slotIndex).toLocaleString()}
          </span>{" "}
          slots
        </span>
      </div>
    </div>
  );
}
