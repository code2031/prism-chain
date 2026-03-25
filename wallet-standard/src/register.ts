/**
 * Wallet Registration — Registers the SolClone wallet with the
 * Wallet Standard global registry so that DApps can discover it.
 *
 * Call `registerSolCloneWallet()` once during your wallet extension's
 * initialization (e.g., in a content script or at app startup).
 */

import { registerWallet } from "@wallet-standard/base";
import type { Wallet } from "@wallet-standard/base";
import {
  SolCloneWallet,
  type KeypairProvider,
  type TransactionSender,
} from "./wallet";

/**
 * Registers the SolClone wallet with the Wallet Standard registry.
 *
 * @param keypairProvider - Provides access to the user's keypair.
 * @param transactionSender - Broadcasts signed transactions to the network.
 * @returns A cleanup function that unregisters the wallet.
 *
 * @example
 * ```ts
 * import { registerSolCloneWallet } from "@solclone/wallet-standard";
 *
 * const unregister = registerSolCloneWallet(myKeypairProvider, myRpcSender);
 *
 * // Later, to unregister:
 * unregister();
 * ```
 */
export function registerSolCloneWallet(
  keypairProvider: KeypairProvider,
  transactionSender: TransactionSender,
): () => void {
  const wallet = new SolCloneWallet(keypairProvider, transactionSender);

  // The Wallet Standard `registerWallet` function adds the wallet to
  // the global `window.__wallets__` registry (or the equivalent shim)
  // and dispatches a `wallet-standard:register` event so DApps that
  // are already listening can pick it up immediately.
  const unregister = registerWallet(wallet as unknown as Wallet);

  return () => {
    if (typeof unregister === "function") {
      unregister();
    }
  };
}

/**
 * Convenience: registers the wallet and also stores a reference on
 * `globalThis` so other SolClone modules can access it directly.
 */
export function registerSolCloneWalletGlobally(
  keypairProvider: KeypairProvider,
  transactionSender: TransactionSender,
): () => void {
  const wallet = new SolCloneWallet(keypairProvider, transactionSender);

  // Expose on globalThis for cross-module access within SolClone.
  (globalThis as any).__solclone_wallet__ = wallet;

  const unregister = registerWallet(wallet as unknown as Wallet);

  return () => {
    if (typeof unregister === "function") {
      unregister();
    }
    delete (globalThis as any).__solclone_wallet__;
  };
}
