"use client";

import { useState } from "react";
import NftCard from "../components/nft-card";

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

const FEATURED_COLLECTIONS = [
  {
    id: "sol-apes",
    name: "SolApes",
    floor: 12.5,
    volume: 48200,
    items: 10000,
  },
  {
    id: "pixel-punks",
    name: "Pixel Punks",
    floor: 5.8,
    volume: 22100,
    items: 5000,
  },
  {
    id: "cosmic-cats",
    name: "Cosmic Cats",
    floor: 3.2,
    volume: 15800,
    items: 8888,
  },
  {
    id: "neon-worlds",
    name: "Neon Worlds",
    floor: 8.4,
    volume: 31500,
    items: 3333,
  },
];

const MOCK_NFTS = [
  {
    id: "1",
    name: "Astro Ape #4291",
    image: "/placeholder-nft.jpg",
    collection: "SolApes",
    priceSol: 14.5,
    priceUsd: 2175,
  },
  {
    id: "2",
    name: "Pixel Punk #782",
    image: "/placeholder-nft.jpg",
    collection: "Pixel Punks",
    priceSol: 6.2,
    priceUsd: 930,
  },
  {
    id: "3",
    name: "Cosmic Cat #1337",
    image: "/placeholder-nft.jpg",
    collection: "Cosmic Cats",
    priceSol: 3.8,
    priceUsd: 570,
    isAuction: true,
    currentBid: 4.1,
    endTime: "2h 14m",
  },
  {
    id: "4",
    name: "Neon City #55",
    image: "/placeholder-nft.jpg",
    collection: "Neon Worlds",
    priceSol: 9.0,
    priceUsd: 1350,
  },
  {
    id: "5",
    name: "Astro Ape #8812",
    image: "/placeholder-nft.jpg",
    collection: "SolApes",
    priceSol: 18.2,
    priceUsd: 2730,
    isAuction: true,
    currentBid: 19.5,
    endTime: "5h 42m",
  },
  {
    id: "6",
    name: "Pixel Punk #2001",
    image: "/placeholder-nft.jpg",
    collection: "Pixel Punks",
    priceSol: 7.1,
    priceUsd: 1065,
  },
  {
    id: "7",
    name: "Void Walker #99",
    image: "/placeholder-nft.jpg",
    collection: "Neon Worlds",
    priceSol: 11.3,
    priceUsd: 1695,
  },
  {
    id: "8",
    name: "Cosmic Cat #420",
    image: "/placeholder-nft.jpg",
    collection: "Cosmic Cats",
    priceSol: 4.5,
    priceUsd: 675,
  },
];

const SORT_OPTIONS = [
  { value: "recent", label: "Recently Listed" },
  { value: "price-low", label: "Price: Low to High" },
  { value: "price-high", label: "Price: High to Low" },
];

const COLLECTIONS_FILTER = [
  "All",
  "SolApes",
  "Pixel Punks",
  "Cosmic Cats",
  "Neon Worlds",
];

export default function MarketplaceHome() {
  const [search, setSearch] = useState("");
  const [selectedCollection, setSelectedCollection] = useState("All");
  const [sortBy, setSortBy] = useState("recent");
  const [priceMin, setPriceMin] = useState("");
  const [priceMax, setPriceMax] = useState("");
  const [carouselIndex, setCarouselIndex] = useState(0);

  // Filter + sort
  const filtered = MOCK_NFTS.filter((nft) => {
    if (selectedCollection !== "All" && nft.collection !== selectedCollection)
      return false;
    if (search && !nft.name.toLowerCase().includes(search.toLowerCase()))
      return false;
    if (priceMin && nft.priceSol < parseFloat(priceMin)) return false;
    if (priceMax && nft.priceSol > parseFloat(priceMax)) return false;
    return true;
  }).sort((a, b) => {
    if (sortBy === "price-low") return a.priceSol - b.priceSol;
    if (sortBy === "price-high") return b.priceSol - a.priceSol;
    return 0;
  });

  const prevSlide = () =>
    setCarouselIndex((i) =>
      i === 0 ? FEATURED_COLLECTIONS.length - 1 : i - 1,
    );
  const nextSlide = () =>
    setCarouselIndex((i) =>
      i === FEATURED_COLLECTIONS.length - 1 ? 0 : i + 1,
    );

  return (
    <div className="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
      {/* ── Hero ── */}
      <section className="mb-14 text-center animate-fade-in">
        <h1 className="text-4xl font-extrabold tracking-tight sm:text-5xl lg:text-6xl">
          <span className="gradient-text">SolMart</span>
        </h1>
        <p className="mx-auto mt-4 max-w-xl text-base text-muted sm:text-lg">
          Discover, collect, and trade premium NFTs on the SolClone blockchain.
          Auctions, instant buys, and creator royalties &mdash; all on-chain.
        </p>
      </section>

      {/* ── Featured Collections Carousel ── */}
      <section className="mb-14">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-bold text-foreground">
            Featured Collections
          </h2>
          <div className="flex gap-2">
            <button
              onClick={prevSlide}
              className="flex h-8 w-8 items-center justify-center rounded-lg border border-card-border text-muted transition-colors hover:border-accent-purple hover:text-foreground"
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <polyline points="15 18 9 12 15 6" />
              </svg>
            </button>
            <button
              onClick={nextSlide}
              className="flex h-8 w-8 items-center justify-center rounded-lg border border-card-border text-muted transition-colors hover:border-accent-purple hover:text-foreground"
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
          </div>
        </div>

        <div className="overflow-hidden rounded-2xl">
          <div
            className="flex transition-transform duration-500 ease-out"
            style={{ transform: `translateX(-${carouselIndex * 100}%)` }}
          >
            {FEATURED_COLLECTIONS.map((col) => (
              <div key={col.id} className="w-full flex-shrink-0 px-1">
                <div className="relative h-52 overflow-hidden rounded-2xl border border-card-border bg-gradient-to-br from-accent-purple/20 via-surface to-accent-green/20 sm:h-64">
                  <div className="absolute inset-0 bg-gradient-to-t from-background/90 via-background/30 to-transparent" />
                  <div className="absolute bottom-0 left-0 right-0 p-6">
                    <h3 className="text-xl font-bold text-foreground">
                      {col.name}
                    </h3>
                    <div className="mt-2 flex gap-6 text-sm">
                      <span className="text-muted">
                        Floor:{" "}
                        <span className="font-semibold text-foreground">
                          {col.floor} SOL
                        </span>
                      </span>
                      <span className="text-muted">
                        Volume:{" "}
                        <span className="font-semibold text-foreground">
                          {col.volume.toLocaleString()} SOL
                        </span>
                      </span>
                      <span className="text-muted">
                        Items:{" "}
                        <span className="font-semibold text-foreground">
                          {col.items.toLocaleString()}
                        </span>
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Dots */}
        <div className="mt-4 flex justify-center gap-2">
          {FEATURED_COLLECTIONS.map((_, i) => (
            <button
              key={i}
              onClick={() => setCarouselIndex(i)}
              className={`h-1.5 rounded-full transition-all ${
                i === carouselIndex
                  ? "w-6 bg-accent-purple"
                  : "w-1.5 bg-card-border"
              }`}
            />
          ))}
        </div>
      </section>

      {/* ── Search / Filter Bar ── */}
      <section className="mb-8">
        <div className="glass-card flex flex-wrap items-center gap-4 rounded-2xl p-4">
          {/* Search */}
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
              placeholder="Search NFTs..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full rounded-xl border border-input-border bg-input-bg py-2.5 pl-10 pr-4 text-sm text-foreground placeholder:text-muted/60 focus:border-accent-purple focus:outline-none"
            />
          </div>

          {/* Collection filter */}
          <div className="flex flex-wrap gap-2">
            {COLLECTIONS_FILTER.map((col) => (
              <button
                key={col}
                onClick={() => setSelectedCollection(col)}
                className={`rounded-lg px-3 py-2 text-xs font-medium transition-colors ${
                  selectedCollection === col
                    ? "bg-accent-purple text-white"
                    : "border border-card-border text-muted hover:border-accent-purple/50 hover:text-foreground"
                }`}
              >
                {col}
              </button>
            ))}
          </div>

          {/* Price range */}
          <div className="flex items-center gap-2">
            <input
              type="number"
              placeholder="Min"
              value={priceMin}
              onChange={(e) => setPriceMin(e.target.value)}
              className="w-20 rounded-lg border border-input-border bg-input-bg px-2 py-2 text-xs text-foreground placeholder:text-muted/60 focus:border-accent-purple focus:outline-none"
            />
            <span className="text-xs text-muted">-</span>
            <input
              type="number"
              placeholder="Max"
              value={priceMax}
              onChange={(e) => setPriceMax(e.target.value)}
              className="w-20 rounded-lg border border-input-border bg-input-bg px-2 py-2 text-xs text-foreground placeholder:text-muted/60 focus:border-accent-purple focus:outline-none"
            />
            <span className="text-xs text-muted">SOL</span>
          </div>

          {/* Sort */}
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value)}
            className="rounded-lg border border-input-border bg-input-bg px-3 py-2.5 text-xs text-foreground focus:border-accent-purple focus:outline-none appearance-none"
          >
            {SORT_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
      </section>

      {/* ── Trending NFTs ── */}
      <section className="mb-14">
        <h2 className="mb-6 text-lg font-bold text-foreground">
          Trending NFTs
        </h2>
        <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {filtered.slice(0, 4).map((nft) => (
            <NftCard key={nft.id} {...nft} />
          ))}
        </div>
      </section>

      {/* ── Recent Listings ── */}
      <section>
        <h2 className="mb-6 text-lg font-bold text-foreground">
          Recent Listings
        </h2>
        {filtered.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-2xl border border-card-border bg-surface/50 py-20">
            <svg
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              className="text-muted"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <p className="mt-4 text-sm text-muted">
              No NFTs match your filters
            </p>
          </div>
        ) : (
          <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
            {filtered.map((nft) => (
              <NftCard key={nft.id} {...nft} />
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
