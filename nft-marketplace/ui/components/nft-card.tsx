"use client";

import Image from "next/image";
import { useState } from "react";

export interface NftCardProps {
  id: string;
  name: string;
  image: string;
  collection: string;
  priceSol: number;
  priceUsd: number;
  isAuction?: boolean;
  currentBid?: number;
  endTime?: string;
  onBuy?: (id: string) => void;
  onBid?: (id: string) => void;
}

export default function NftCard({
  id,
  name,
  image,
  collection,
  priceSol,
  priceUsd,
  isAuction = false,
  currentBid,
  endTime,
  onBuy,
  onBid,
}: NftCardProps) {
  const [imgError, setImgError] = useState(false);

  return (
    <div className="group glass-card hover-lift rounded-2xl overflow-hidden animate-fade-in">
      {/* Image */}
      <div className="relative aspect-square overflow-hidden bg-surface">
        {imgError ? (
          <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-accent-purple/20 to-accent-green/20">
            <svg
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              className="text-muted"
            >
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <polyline points="21 15 16 10 5 21" />
            </svg>
          </div>
        ) : (
          <Image
            src={image}
            alt={name}
            fill
            className="object-cover transition-transform duration-500 group-hover:scale-110"
            onError={() => setImgError(true)}
            sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 25vw"
          />
        )}

        {/* Collection badge */}
        <div className="absolute left-3 top-3">
          <span className="inline-flex items-center gap-1 rounded-full bg-surface/80 px-2.5 py-1 text-xs font-medium text-accent-purple backdrop-blur-sm">
            <svg
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="currentColor"
            >
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {collection}
          </span>
        </div>

        {/* Auction timer badge */}
        {isAuction && endTime && (
          <div className="absolute right-3 top-3">
            <span className="inline-flex items-center gap-1 rounded-full bg-accent-pink/90 px-2.5 py-1 text-xs font-semibold text-white backdrop-blur-sm animate-countdown">
              <svg
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <circle cx="12" cy="12" r="10" />
                <polyline points="12 6 12 12 16 14" />
              </svg>
              {endTime}
            </span>
          </div>
        )}
      </div>

      {/* Details */}
      <div className="p-4">
        <h3 className="truncate text-sm font-semibold text-foreground">
          {name}
        </h3>

        <div className="mt-3 flex items-end justify-between">
          <div>
            <p className="text-xs text-muted">
              {isAuction ? "Current Bid" : "Price"}
            </p>
            <p className="mt-0.5 text-base font-bold text-foreground">
              {isAuction && currentBid !== undefined
                ? currentBid.toFixed(2)
                : priceSol.toFixed(2)}{" "}
              <span className="text-xs font-normal text-muted">SOL</span>
            </p>
            <p className="text-xs text-muted">${priceUsd.toLocaleString()}</p>
          </div>

          {isAuction ? (
            <button
              onClick={() => onBid?.(id)}
              className="rounded-lg bg-accent-pink/20 px-4 py-2 text-xs font-semibold text-accent-pink transition-colors hover:bg-accent-pink/30"
            >
              Place Bid
            </button>
          ) : (
            <button
              onClick={() => onBuy?.(id)}
              className="rounded-lg bg-gradient-to-r from-accent-purple to-accent-green px-4 py-2 text-xs font-semibold text-white transition-opacity hover:opacity-90"
            >
              Buy Now
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
