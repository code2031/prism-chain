"use client";

import Image from "next/image";
import { useState } from "react";

export interface CollectionBannerProps {
  name: string;
  description?: string;
  bannerImage: string;
  avatarImage: string;
  floorPrice: number;
  totalVolume: number;
  itemsCount: number;
  ownersCount?: number;
  isVerified?: boolean;
}

export default function CollectionBanner({
  name,
  description,
  bannerImage,
  avatarImage,
  floorPrice,
  totalVolume,
  itemsCount,
  ownersCount,
  isVerified = false,
}: CollectionBannerProps) {
  const [bannerError, setBannerError] = useState(false);
  const [avatarError, setAvatarError] = useState(false);

  return (
    <div className="animate-fade-in">
      {/* Banner */}
      <div className="relative h-48 w-full overflow-hidden rounded-2xl sm:h-64 lg:h-72">
        {bannerError ? (
          <div className="h-full w-full bg-gradient-to-r from-accent-purple/30 via-surface to-accent-green/30" />
        ) : (
          <Image
            src={bannerImage}
            alt={`${name} banner`}
            fill
            priority
            className="object-cover"
            onError={() => setBannerError(true)}
            sizes="100vw"
          />
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-background via-background/40 to-transparent" />
      </div>

      {/* Avatar + Info */}
      <div className="relative mx-auto max-w-4xl px-4 sm:px-6">
        {/* Avatar */}
        <div className="-mt-16 mb-4 flex justify-center sm:-mt-20">
          <div className="relative h-28 w-28 overflow-hidden rounded-2xl border-4 border-background shadow-lg sm:h-36 sm:w-36">
            {avatarError ? (
              <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-accent-purple to-accent-green">
                <span className="text-3xl font-bold text-white">
                  {name.charAt(0)}
                </span>
              </div>
            ) : (
              <Image
                src={avatarImage}
                alt={name}
                fill
                className="object-cover"
                onError={() => setAvatarError(true)}
                sizes="144px"
              />
            )}
          </div>
        </div>

        {/* Name + verified */}
        <div className="flex flex-col items-center gap-2 text-center">
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold text-foreground sm:text-3xl">
              {name}
            </h1>
            {isVerified && (
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="currentColor"
                className="text-accent-purple"
              >
                <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            )}
          </div>

          {description && (
            <p className="max-w-xl text-sm text-muted">{description}</p>
          )}
        </div>

        {/* Stats */}
        <div className="mt-6 flex flex-wrap items-center justify-center gap-6 sm:gap-10">
          <Stat label="Floor Price" value={`${floorPrice.toFixed(2)} SOL`} />
          <Stat
            label="Total Volume"
            value={`${totalVolume.toLocaleString()} SOL`}
          />
          <Stat label="Items" value={itemsCount.toLocaleString()} />
          {ownersCount !== undefined && (
            <Stat label="Owners" value={ownersCount.toLocaleString()} />
          )}
        </div>
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="text-center">
      <p className="text-lg font-bold text-foreground sm:text-xl">{value}</p>
      <p className="text-xs text-muted">{label}</p>
    </div>
  );
}
