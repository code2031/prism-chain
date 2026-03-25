"use client";

import Image from "next/image";
import { useState } from "react";

interface Attribute {
  trait: string;
  value: string;
  rarity?: string;
}

interface ActivityItem {
  event: "Sale" | "List" | "Transfer" | "Bid" | "Mint";
  price?: number;
  from: string;
  to: string;
  date: string;
}

export interface NftDetailProps {
  id: string;
  name: string;
  image: string;
  collection: string;
  collectionVerified?: boolean;
  owner: string;
  priceSol: number;
  priceUsd: number;
  description?: string;
  attributes: Attribute[];
  activity: ActivityItem[];
  onBuy?: (id: string) => void;
  onMakeOffer?: (id: string) => void;
}

export default function NftDetail({
  id,
  name,
  image,
  collection,
  collectionVerified = false,
  owner,
  priceSol,
  priceUsd,
  description,
  attributes,
  activity,
  onBuy,
  onMakeOffer,
}: NftDetailProps) {
  const [imgError, setImgError] = useState(false);

  const eventColors: Record<string, string> = {
    Sale: "text-accent-green",
    List: "text-accent-purple",
    Transfer: "text-blue-400",
    Bid: "text-accent-pink",
    Mint: "text-yellow-400",
  };

  return (
    <div className="mx-auto max-w-6xl px-4 py-8 sm:px-6 lg:px-8 animate-fade-in">
      <div className="grid gap-10 lg:grid-cols-2">
        {/* ── Left: Image ── */}
        <div className="relative aspect-square overflow-hidden rounded-3xl border border-card-border bg-surface">
          {imgError ? (
            <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-accent-purple/10 to-accent-green/10">
              <svg
                width="80"
                height="80"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
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
              priority
              className="object-cover"
              onError={() => setImgError(true)}
              sizes="(max-width: 1024px) 100vw, 50vw"
            />
          )}
        </div>

        {/* ── Right: Details ── */}
        <div className="flex flex-col gap-6">
          {/* Collection + name */}
          <div>
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-accent-purple">
                {collection}
              </span>
              {collectionVerified && (
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  className="text-accent-purple"
                >
                  <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              )}
            </div>
            <h1 className="mt-1 text-3xl font-bold text-foreground">{name}</h1>
          </div>

          {/* Owner */}
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-full bg-gradient-to-br from-accent-purple to-accent-green" />
            <div>
              <p className="text-xs text-muted">Owned by</p>
              <p className="text-sm font-medium text-foreground font-mono">
                {owner.slice(0, 4)}...{owner.slice(-4)}
              </p>
            </div>
          </div>

          {/* Price box */}
          <div className="glass-card rounded-2xl p-6">
            <p className="text-xs text-muted">Current Price</p>
            <div className="mt-2 flex items-baseline gap-3">
              <span className="text-3xl font-bold text-foreground">
                {priceSol.toFixed(2)} SOL
              </span>
              <span className="text-sm text-muted">
                ${priceUsd.toLocaleString()}
              </span>
            </div>

            <div className="mt-5 flex gap-3">
              <button
                onClick={() => onBuy?.(id)}
                className="flex-1 rounded-xl bg-gradient-to-r from-accent-purple to-accent-green py-3 text-center text-sm font-semibold text-white transition-opacity hover:opacity-90"
              >
                Buy Now
              </button>
              <button
                onClick={() => onMakeOffer?.(id)}
                className="flex-1 rounded-xl border border-accent-purple/50 py-3 text-center text-sm font-semibold text-accent-purple transition-colors hover:bg-accent-purple/10"
              >
                Make Offer
              </button>
            </div>
          </div>

          {/* Description */}
          {description && (
            <div>
              <h3 className="text-sm font-semibold text-foreground">
                Description
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-muted">
                {description}
              </p>
            </div>
          )}

          {/* Attributes grid */}
          {attributes.length > 0 && (
            <div>
              <h3 className="text-sm font-semibold text-foreground">
                Attributes
              </h3>
              <div className="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-3">
                {attributes.map((attr) => (
                  <div
                    key={attr.trait}
                    className="rounded-xl border border-accent-purple/20 bg-accent-purple/5 p-3 text-center"
                  >
                    <p className="text-[10px] font-semibold uppercase tracking-wider text-accent-purple">
                      {attr.trait}
                    </p>
                    <p className="mt-1 text-sm font-medium text-foreground">
                      {attr.value}
                    </p>
                    {attr.rarity && (
                      <p className="mt-0.5 text-[10px] text-muted">
                        {attr.rarity}
                      </p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Activity history */}
          {activity.length > 0 && (
            <div>
              <h3 className="text-sm font-semibold text-foreground">
                Activity
              </h3>
              <div className="mt-3 divide-y divide-card-border/30 rounded-xl border border-card-border bg-surface/50">
                {activity.map((item, i) => (
                  <div
                    key={i}
                    className="flex items-center justify-between px-4 py-3"
                  >
                    <div className="flex items-center gap-3">
                      <span
                        className={`text-xs font-semibold ${eventColors[item.event] || "text-muted"}`}
                      >
                        {item.event}
                      </span>
                      <span className="text-xs text-muted font-mono">
                        {item.from.slice(0, 4)}...{item.from.slice(-4)}
                      </span>
                      <svg
                        width="12"
                        height="12"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        className="text-muted"
                      >
                        <line x1="5" y1="12" x2="19" y2="12" />
                        <polyline points="12 5 19 12 12 19" />
                      </svg>
                      <span className="text-xs text-muted font-mono">
                        {item.to.slice(0, 4)}...{item.to.slice(-4)}
                      </span>
                    </div>
                    <div className="text-right">
                      {item.price !== undefined && (
                        <p className="text-xs font-medium text-foreground">
                          {item.price.toFixed(2)} SOL
                        </p>
                      )}
                      <p className="text-[10px] text-muted">{item.date}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
