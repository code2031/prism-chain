"use client";

import { useState } from "react";

const COLLECTIONS = [
  {
    id: "sol-apes",
    name: "SolApes",
    avatar: "",
    floorPrice: 12.5,
    volume: 48200,
    items: 10000,
    owners: 4200,
    change24h: 5.3,
    verified: true,
  },
  {
    id: "pixel-punks",
    name: "Pixel Punks",
    avatar: "",
    floorPrice: 5.8,
    volume: 22100,
    items: 5000,
    owners: 2800,
    change24h: -2.1,
    verified: true,
  },
  {
    id: "cosmic-cats",
    name: "Cosmic Cats",
    avatar: "",
    floorPrice: 3.2,
    volume: 15800,
    items: 8888,
    owners: 5100,
    change24h: 12.7,
    verified: true,
  },
  {
    id: "neon-worlds",
    name: "Neon Worlds",
    avatar: "",
    floorPrice: 8.4,
    volume: 31500,
    items: 3333,
    owners: 1900,
    change24h: 0.8,
    verified: true,
  },
  {
    id: "sol-skulls",
    name: "SolSkulls",
    avatar: "",
    floorPrice: 2.1,
    volume: 9400,
    items: 6666,
    owners: 3200,
    change24h: -5.4,
    verified: false,
  },
  {
    id: "dream-scapes",
    name: "DreamScapes",
    avatar: "",
    floorPrice: 15.3,
    volume: 52000,
    items: 2222,
    owners: 1100,
    change24h: 8.9,
    verified: true,
  },
  {
    id: "robo-realm",
    name: "RoboRealm",
    avatar: "",
    floorPrice: 1.8,
    volume: 6200,
    items: 10000,
    owners: 6500,
    change24h: 1.2,
    verified: false,
  },
  {
    id: "abstract-minds",
    name: "Abstract Minds",
    avatar: "",
    floorPrice: 4.7,
    volume: 18900,
    items: 4444,
    owners: 2400,
    change24h: -0.3,
    verified: true,
  },
];

type SortKey = "volume" | "floorPrice" | "items" | "change24h";

export default function CollectionsPage() {
  const [sortKey, setSortKey] = useState<SortKey>("volume");
  const [search, setSearch] = useState("");

  const filtered = COLLECTIONS.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase()),
  ).sort((a, b) => {
    if (sortKey === "change24h") return b.change24h - a.change24h;
    return (b[sortKey] as number) - (a[sortKey] as number);
  });

  return (
    <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      {/* Header */}
      <div className="mb-8 animate-fade-in">
        <h1 className="text-3xl font-bold text-foreground">Collections</h1>
        <p className="mt-2 text-sm text-muted">
          Browse verified and trending NFT collections on SolMart
        </p>
      </div>

      {/* Toolbar */}
      <div className="mb-6 flex flex-wrap items-center gap-4">
        <div className="relative flex-1 min-w-[200px]">
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            className="absolute left-3 top-1/2 -translate-y-1/2 text-muted"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="text"
            placeholder="Search collections..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full rounded-xl border border-input-border bg-input-bg py-2.5 pl-10 pr-4 text-sm text-foreground placeholder:text-muted/60 focus:border-accent-purple focus:outline-none"
          />
        </div>

        <div className="flex gap-2">
          {(
            [
              { key: "volume", label: "Volume" },
              { key: "floorPrice", label: "Floor" },
              { key: "items", label: "Items" },
              { key: "change24h", label: "24h %" },
            ] as { key: SortKey; label: string }[]
          ).map((opt) => (
            <button
              key={opt.key}
              onClick={() => setSortKey(opt.key)}
              className={`rounded-lg px-3 py-2 text-xs font-medium transition-colors ${
                sortKey === opt.key
                  ? "bg-accent-purple text-white"
                  : "border border-card-border text-muted hover:border-accent-purple/50 hover:text-foreground"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* Collection grid */}
      <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {filtered.map((col, idx) => (
          <div
            key={col.id}
            className="glass-card hover-lift rounded-2xl overflow-hidden animate-fade-in"
            style={{ animationDelay: `${idx * 60}ms` }}
          >
            {/* Banner gradient */}
            <div className="relative h-28 bg-gradient-to-br from-accent-purple/30 via-surface to-accent-green/30">
              <div className="absolute inset-0 bg-gradient-to-t from-card-bg to-transparent" />
              {/* Rank badge */}
              <div className="absolute left-3 top-3">
                <span className="flex h-7 w-7 items-center justify-center rounded-full bg-surface/80 text-xs font-bold text-foreground backdrop-blur-sm">
                  {idx + 1}
                </span>
              </div>
            </div>

            {/* Avatar */}
            <div className="relative -mt-10 flex justify-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-xl border-4 border-card-bg bg-gradient-to-br from-accent-purple to-accent-green shadow-lg">
                <span className="text-lg font-bold text-white">
                  {col.name.charAt(0)}
                </span>
              </div>
            </div>

            {/* Info */}
            <div className="p-4 pt-3 text-center">
              <div className="flex items-center justify-center gap-1">
                <h3 className="text-sm font-semibold text-foreground">
                  {col.name}
                </h3>
                {col.verified && (
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    className="text-accent-purple"
                  >
                    <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                )}
              </div>

              <div className="mt-4 grid grid-cols-2 gap-3">
                <div>
                  <p className="text-xs text-muted">Floor</p>
                  <p className="text-sm font-semibold text-foreground">
                    {col.floorPrice} SOL
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted">Volume</p>
                  <p className="text-sm font-semibold text-foreground">
                    {col.volume.toLocaleString()}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted">Items</p>
                  <p className="text-sm font-semibold text-foreground">
                    {col.items.toLocaleString()}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-muted">24h</p>
                  <p
                    className={`text-sm font-semibold ${col.change24h >= 0 ? "text-accent-green" : "text-accent-pink"}`}
                  >
                    {col.change24h >= 0 ? "+" : ""}
                    {col.change24h.toFixed(1)}%
                  </p>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div className="flex flex-col items-center justify-center rounded-2xl border border-card-border bg-surface/50 py-20">
          <p className="text-sm text-muted">No collections found</p>
        </div>
      )}
    </div>
  );
}
