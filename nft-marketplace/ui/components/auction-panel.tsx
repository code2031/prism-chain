"use client";

import { useCallback, useEffect, useState } from "react";

interface BidHistoryItem {
  bidder: string;
  amount: number;
  time: string;
}

export interface AuctionPanelProps {
  auctionId: string;
  currentBid: number;
  /** ISO-8601 end time */
  endTime: string;
  reservePrice?: number;
  bidHistory: BidHistoryItem[];
  onPlaceBid?: (auctionId: string, amount: number) => void;
}

function formatTimeLeft(ms: number): {
  days: number;
  hours: number;
  minutes: number;
  seconds: number;
} {
  if (ms <= 0) return { days: 0, hours: 0, minutes: 0, seconds: 0 };
  const seconds = Math.floor((ms / 1000) % 60);
  const minutes = Math.floor((ms / (1000 * 60)) % 60);
  const hours = Math.floor((ms / (1000 * 60 * 60)) % 24);
  const days = Math.floor(ms / (1000 * 60 * 60 * 24));
  return { days, hours, minutes, seconds };
}

export default function AuctionPanel({
  auctionId,
  currentBid,
  endTime,
  reservePrice,
  bidHistory,
  onPlaceBid,
}: AuctionPanelProps) {
  const [timeLeft, setTimeLeft] = useState(() =>
    formatTimeLeft(new Date(endTime).getTime() - Date.now()),
  );
  const [bidAmount, setBidAmount] = useState("");
  const isEnded =
    timeLeft.days === 0 &&
    timeLeft.hours === 0 &&
    timeLeft.minutes === 0 &&
    timeLeft.seconds === 0;

  useEffect(() => {
    const timer = setInterval(() => {
      const remaining = new Date(endTime).getTime() - Date.now();
      setTimeLeft(formatTimeLeft(remaining));
      if (remaining <= 0) clearInterval(timer);
    }, 1000);
    return () => clearInterval(timer);
  }, [endTime]);

  const handleBid = useCallback(() => {
    const amount = parseFloat(bidAmount);
    if (!isNaN(amount) && amount > currentBid) {
      onPlaceBid?.(auctionId, amount);
      setBidAmount("");
    }
  }, [bidAmount, currentBid, auctionId, onPlaceBid]);

  const minBid = (currentBid + 0.01).toFixed(2);

  return (
    <div className="glass-card rounded-2xl p-6 animate-fade-in">
      {/* ── Current bid ── */}
      <div className="mb-6">
        <p className="text-xs text-muted">Current Bid</p>
        <p className="mt-1 text-3xl font-bold text-foreground">
          {currentBid.toFixed(2)}{" "}
          <span className="text-base font-normal text-muted">SOL</span>
        </p>
        {reservePrice !== undefined && (
          <p className="mt-1 text-xs text-muted">
            Reserve:{" "}
            <span
              className={
                currentBid >= reservePrice
                  ? "text-accent-green"
                  : "text-accent-pink"
              }
            >
              {currentBid >= reservePrice ? "Met" : "Not met"}
            </span>
          </p>
        )}
      </div>

      {/* ── Countdown ── */}
      <div className="mb-6">
        <p className="mb-2 text-xs text-muted">Time Remaining</p>
        <div className="grid grid-cols-4 gap-3">
          {(["days", "hours", "minutes", "seconds"] as const).map((unit) => (
            <div
              key={unit}
              className="flex flex-col items-center rounded-xl border border-card-border bg-surface/50 py-3"
            >
              <span
                className={`text-2xl font-bold tabular-nums ${isEnded ? "text-accent-pink" : "text-foreground"}`}
              >
                {String(timeLeft[unit]).padStart(2, "0")}
              </span>
              <span className="mt-0.5 text-[10px] uppercase tracking-wider text-muted">
                {unit}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* ── Place bid ── */}
      {!isEnded && (
        <div className="mb-6">
          <div className="flex gap-3">
            <div className="relative flex-1">
              <input
                type="number"
                step="0.01"
                min={minBid}
                placeholder={`Min ${minBid} SOL`}
                value={bidAmount}
                onChange={(e) => setBidAmount(e.target.value)}
                className="w-full rounded-xl border border-input-border bg-input-bg px-4 py-3 pr-14 text-sm text-foreground placeholder:text-muted/60 focus:border-accent-purple focus:outline-none focus:ring-1 focus:ring-accent-purple/50"
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-xs text-muted">
                SOL
              </span>
            </div>
            <button
              type="button"
              onClick={handleBid}
              disabled={
                !bidAmount || parseFloat(bidAmount) <= currentBid
              }
              className="rounded-xl bg-accent-pink px-6 py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90 disabled:opacity-40"
            >
              Place Bid
            </button>
          </div>
        </div>
      )}

      {isEnded && (
        <div className="mb-6 rounded-xl border border-accent-pink/30 bg-accent-pink/10 p-4 text-center">
          <p className="text-sm font-semibold text-accent-pink">
            Auction Ended
          </p>
        </div>
      )}

      {/* ── Bid history ── */}
      {bidHistory.length > 0 && (
        <div>
          <p className="mb-3 text-xs font-semibold text-foreground">
            Bid History
          </p>
          <div className="divide-y divide-card-border/30 rounded-xl border border-card-border bg-surface/50">
            {bidHistory.map((bid, i) => (
              <div
                key={i}
                className="flex items-center justify-between px-4 py-3"
              >
                <div className="flex items-center gap-3">
                  <div className="h-7 w-7 rounded-full bg-gradient-to-br from-accent-purple to-accent-pink" />
                  <span className="text-xs font-medium text-foreground font-mono">
                    {bid.bidder.slice(0, 4)}...{bid.bidder.slice(-4)}
                  </span>
                </div>
                <div className="text-right">
                  <p className="text-xs font-semibold text-foreground">
                    {bid.amount.toFixed(2)} SOL
                  </p>
                  <p className="text-[10px] text-muted">{bid.time}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
