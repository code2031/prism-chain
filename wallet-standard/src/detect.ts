/**
 * Wallet Detection — Discovers all Wallet Standard wallets registered
 * in the current environment. Useful for building wallet selection
 * dialogs in DApps.
 */

import type { Wallet, WalletIcon } from "@wallet-standard/base";

// ── Types ───────────────────────────────────────────────────────────────────

/** Summary of a detected wallet, suitable for rendering in UI. */
export interface DetectedWallet {
  /** Human-readable wallet name (e.g., "Phantom", "SolClone"). */
  name: string;
  /** Wallet icon as a data URI. */
  icon: WalletIcon | undefined;
  /** Chains the wallet supports (e.g., "solana:mainnet", "solclone:devnet"). */
  chains: readonly string[];
  /** Feature identifiers the wallet exposes. */
  features: string[];
  /** Number of accounts currently connected. */
  accountCount: number;
  /** Reference to the raw Wallet Standard wallet object. */
  raw: Wallet;
}

/** Options for the detect call. */
export interface DetectWalletsOptions {
  /**
   * If provided, only wallets that support at least one of these chains
   * will be returned.
   */
  chains?: string[];
  /**
   * If provided, only wallets that expose all of these features will be
   * returned.
   */
  requiredFeatures?: string[];
}

// ── Registry Access ─────────────────────────────────────────────────────────

/**
 * Reads the Wallet Standard registry from `globalThis`.
 *
 * The Wallet Standard stores registered wallets in
 * `window.__wallets__` (an array-like pushed to via `registerWallet`).
 * We also fire a `wallet-standard:app-ready` event to prompt any
 * lazy-registering wallets to register now.
 */
function getRegisteredWallets(): Wallet[] {
  const wallets: Wallet[] = [];

  // Notify wallet extensions that a DApp is ready.
  if (typeof window !== "undefined") {
    const event = new Event("wallet-standard:app-ready");
    window.dispatchEvent(event);
  }

  // Read from the standard callback-based API if available.
  const global = globalThis as any;
  const register = global["wallet-standard:register"];

  if (typeof register === "function") {
    register((...args: any[]) => {
      // The callback receives a `register` function; we call it with
      // a collector.
    });
  }

  // Read from the older array-based registry.
  const registry: any[] | undefined = global.__wallets__;
  if (Array.isArray(registry)) {
    for (const entry of registry) {
      // Each entry might be a wallet directly or { wallet }.
      const w: Wallet | undefined = entry?.wallet ?? entry;
      if (w && typeof w.name === "string" && w.features) {
        wallets.push(w);
      }
    }
  }

  // Also check for the callback-based `get` pattern used by
  // @wallet-standard/app.
  if (global.__wallet_standard__?.get) {
    const fetched: Wallet[] = global.__wallet_standard__.get();
    for (const w of fetched) {
      if (!wallets.includes(w)) {
        wallets.push(w);
      }
    }
  }

  return wallets;
}

// ── Public API ──────────────────────────────────────────────────────────────

/**
 * Detects all Wallet Standard wallets currently registered.
 *
 * @param options - Optional filters for chains and features.
 * @returns An array of `DetectedWallet` summaries sorted alphabetically by name.
 *
 * @example
 * ```ts
 * import { detectWallets } from "@solclone/wallet-standard";
 *
 * const wallets = detectWallets({ chains: ["solclone:mainnet"] });
 * wallets.forEach(w => console.log(w.name, w.icon, w.chains));
 * ```
 */
export function detectWallets(
  options: DetectWalletsOptions = {},
): DetectedWallet[] {
  const raw = getRegisteredWallets();

  let results: DetectedWallet[] = raw.map((wallet) => ({
    name: wallet.name,
    icon: wallet.icon,
    chains: wallet.chains,
    features: Object.keys(wallet.features),
    accountCount: wallet.accounts.length,
    raw: wallet,
  }));

  // Filter by chains if specified.
  if (options.chains && options.chains.length > 0) {
    const allowed = new Set(options.chains);
    results = results.filter((w) =>
      w.chains.some((c) => allowed.has(c)),
    );
  }

  // Filter by required features if specified.
  if (options.requiredFeatures && options.requiredFeatures.length > 0) {
    const required = options.requiredFeatures;
    results = results.filter((w) =>
      required.every((f) => w.features.includes(f)),
    );
  }

  // Sort alphabetically by name.
  results.sort((a, b) => a.name.localeCompare(b.name));

  return results;
}

/**
 * Listens for newly registered wallets and invokes the callback.
 * Returns a cleanup function to stop listening.
 *
 * @example
 * ```ts
 * const stop = onWalletRegistered((wallet) => {
 *   console.log("New wallet:", wallet.name);
 * });
 * ```
 */
export function onWalletRegistered(
  callback: (wallet: DetectedWallet) => void,
): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  const handler = (event: Event) => {
    const detail = (event as CustomEvent).detail;
    if (detail && detail.name && detail.features) {
      callback({
        name: detail.name,
        icon: detail.icon,
        chains: detail.chains ?? [],
        features: Object.keys(detail.features ?? {}),
        accountCount: detail.accounts?.length ?? 0,
        raw: detail,
      });
    }
  };

  window.addEventListener("wallet-standard:register", handler);

  return () => {
    window.removeEventListener("wallet-standard:register", handler);
  };
}
