/**
 * SolCloneWalletAdapter — Bridges the SolClone Wallet Standard wallet
 * to the @solana/wallet-adapter ecosystem.
 *
 * This allows DApps using `@solana/wallet-adapter-react` (the most
 * popular Solana wallet integration library) to connect to SolClone
 * without any SolClone-specific code.
 */

import type { Wallet, WalletAccount } from "@wallet-standard/base";
import type {
  StandardConnectFeature,
  StandardDisconnectFeature,
} from "@wallet-standard/features";
import type {
  SolanaSignTransactionFeature,
  SolanaSignAndSendTransactionFeature,
  SolanaSignMessageFeature,
} from "@solana/wallet-standard-features";
import {
  SolCloneConnect,
  SolCloneDisconnect,
  SolCloneSignTransaction,
  SolCloneSignAndSendTransaction,
  SolCloneSignMessage,
} from "./features";

// ── Types ───────────────────────────────────────────────────────────────────

/** Minimal interface matching @solana/wallet-adapter-base WalletAdapter. */
export interface WalletAdapterInterface {
  name: string;
  icon: string;
  url: string;
  publicKey: Uint8Array | null;
  connected: boolean;
  connecting: boolean;
  readyState: "Installed" | "NotDetected" | "Loadable" | "Unsupported";

  connect(): Promise<void>;
  disconnect(): Promise<void>;
  signTransaction(transaction: Uint8Array): Promise<Uint8Array>;
  signAllTransactions(transactions: Uint8Array[]): Promise<Uint8Array[]>;
  signMessage(message: Uint8Array): Promise<Uint8Array>;
  sendTransaction(transaction: Uint8Array): Promise<string>;

  on(event: string, callback: (...args: any[]) => void): void;
  off(event: string, callback: (...args: any[]) => void): void;
}

/** Event callback map. */
type EventMap = Record<string, Set<(...args: any[]) => void>>;

// ── Adapter ─────────────────────────────────────────────────────────────────

export class SolCloneWalletAdapter implements WalletAdapterInterface {
  readonly name = "SolClone";
  readonly icon: string;
  readonly url = "https://solclone.io";

  #wallet: Wallet;
  #account: WalletAccount | null = null;
  #connecting = false;
  #events: EventMap = {};

  get publicKey(): Uint8Array | null {
    return this.#account?.publicKey ?? null;
  }

  get connected(): boolean {
    return this.#account !== null;
  }

  get connecting(): boolean {
    return this.#connecting;
  }

  get readyState(): "Installed" | "NotDetected" {
    return this.#wallet ? "Installed" : "NotDetected";
  }

  constructor(wallet: Wallet) {
    this.#wallet = wallet;
    this.icon =
      typeof wallet.icon === "string"
        ? wallet.icon
        : "data:image/svg+xml;base64,";
  }

  // ── Connect / Disconnect ──────────────────────────────────────────────

  async connect(): Promise<void> {
    if (this.connected) return;

    this.#connecting = true;
    this.#emit("connecting");

    try {
      const connectFeature = this.#wallet.features[
        SolCloneConnect
      ] as StandardConnectFeature[typeof SolCloneConnect] | undefined;

      if (!connectFeature) {
        throw new Error("SolClone wallet does not support connect.");
      }

      const result = await connectFeature.connect();
      const accounts = result.accounts;

      if (!accounts || accounts.length === 0) {
        throw new Error("No accounts returned from SolClone wallet.");
      }

      this.#account = accounts[0];
      this.#emit("connect", this.#account.publicKey);
    } catch (error) {
      this.#emit("error", error);
      throw error;
    } finally {
      this.#connecting = false;
    }
  }

  async disconnect(): Promise<void> {
    if (!this.connected) return;

    try {
      const disconnectFeature = this.#wallet.features[
        SolCloneDisconnect
      ] as StandardDisconnectFeature[typeof SolCloneDisconnect] | undefined;

      if (disconnectFeature) {
        await disconnectFeature.disconnect();
      }
    } finally {
      this.#account = null;
      this.#emit("disconnect");
    }
  }

  // ── Sign Transaction ──────────────────────────────────────────────────

  async signTransaction(transaction: Uint8Array): Promise<Uint8Array> {
    this.#requireConnected();

    const feature = this.#wallet.features[
      SolCloneSignTransaction
    ] as SolanaSignTransactionFeature[typeof SolCloneSignTransaction] | undefined;

    if (!feature) {
      throw new Error("SolClone wallet does not support signTransaction.");
    }

    const [result] = await feature.signTransaction({
      transaction,
      account: this.#account!,
    });

    return result.signedTransaction;
  }

  async signAllTransactions(
    transactions: Uint8Array[],
  ): Promise<Uint8Array[]> {
    this.#requireConnected();

    const feature = this.#wallet.features[
      SolCloneSignTransaction
    ] as SolanaSignTransactionFeature[typeof SolCloneSignTransaction] | undefined;

    if (!feature) {
      throw new Error("SolClone wallet does not support signTransaction.");
    }

    const inputs = transactions.map((transaction) => ({
      transaction,
      account: this.#account!,
    }));

    const results = await feature.signTransaction(...inputs);
    return results.map((r) => r.signedTransaction);
  }

  // ── Sign Message ──────────────────────────────────────────────────────

  async signMessage(message: Uint8Array): Promise<Uint8Array> {
    this.#requireConnected();

    const feature = this.#wallet.features[
      SolCloneSignMessage
    ] as SolanaSignMessageFeature[typeof SolCloneSignMessage] | undefined;

    if (!feature) {
      throw new Error("SolClone wallet does not support signMessage.");
    }

    const [result] = await feature.signMessage({
      message,
      account: this.#account!,
    });

    return result.signature;
  }

  // ── Send Transaction ──────────────────────────────────────────────────

  async sendTransaction(transaction: Uint8Array): Promise<string> {
    this.#requireConnected();

    const feature = this.#wallet.features[
      SolCloneSignAndSendTransaction
    ] as SolanaSignAndSendTransactionFeature[typeof SolCloneSignAndSendTransaction] | undefined;

    if (!feature) {
      throw new Error(
        "SolClone wallet does not support signAndSendTransaction.",
      );
    }

    const [result] = await feature.signAndSendTransaction({
      transaction,
      account: this.#account!,
    });

    // Convert the signature bytes to a base58 string.
    const bs58 = await import("bs58");
    return bs58.default.encode(result.signature);
  }

  // ── Event Emitter ─────────────────────────────────────────────────────

  on(event: string, callback: (...args: any[]) => void): void {
    if (!this.#events[event]) {
      this.#events[event] = new Set();
    }
    this.#events[event].add(callback);
  }

  off(event: string, callback: (...args: any[]) => void): void {
    this.#events[event]?.delete(callback);
  }

  #emit(event: string, ...args: any[]): void {
    const listeners = this.#events[event];
    if (listeners) {
      for (const listener of listeners) {
        try {
          listener(...args);
        } catch {
          // Swallow listener errors.
        }
      }
    }
  }

  // ── Helpers ───────────────────────────────────────────────────────────

  #requireConnected(): void {
    if (!this.connected || !this.#account) {
      throw new Error("Wallet is not connected. Call connect() first.");
    }
  }
}

/**
 * Convenience factory: finds the SolClone wallet in the Wallet Standard
 * registry and returns a ready-to-use adapter.
 */
export function createSolCloneAdapter(): SolCloneWalletAdapter | null {
  const global = globalThis as any;

  // Try the direct reference first.
  if (global.__solclone_wallet__) {
    return new SolCloneWalletAdapter(global.__solclone_wallet__);
  }

  // Fall back to scanning the registry.
  const registry: any[] | undefined = global.__wallets__;
  if (Array.isArray(registry)) {
    for (const entry of registry) {
      const w: Wallet | undefined = entry?.wallet ?? entry;
      if (w && w.name === "SolClone") {
        return new SolCloneWalletAdapter(w);
      }
    }
  }

  return null;
}
