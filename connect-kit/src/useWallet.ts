/**
 * useWallet — Custom hook wrapping the wallet adapter for SolClone DApps.
 *
 * Provides a simplified, unified interface for wallet interactions
 * regardless of which wallet the user connected with.
 */

import { useState, useEffect, useCallback, useRef } from "react";
import type { DetectedWallet } from "@solclone/wallet-standard";

// ── Types ───────────────────────────────────────────────────────────────────

export interface UseWalletReturn {
  /** Whether a wallet is currently connected. */
  connected: boolean;
  /** Whether a connection is in progress. */
  connecting: boolean;
  /** The connected wallet's public key (32 bytes), or null. */
  publicKey: Uint8Array | null;
  /** The connected wallet's base58-encoded address, or null. */
  address: string | null;
  /** The wallet's balance in lamports, or null if not fetched. */
  balance: number | null;
  /** Name of the connected wallet (e.g., "Phantom", "SolClone"). */
  walletName: string | null;
  /** Icon of the connected wallet as a data URI. */
  walletIcon: string | null;

  /** Connect to the specified wallet. */
  connect: (wallet: DetectedWallet) => Promise<void>;
  /** Disconnect the current wallet. */
  disconnect: () => Promise<void>;
  /** Sign a transaction (returns the signed bytes). */
  signTransaction: (transaction: Uint8Array) => Promise<Uint8Array>;
  /** Sign an arbitrary message (returns the signature bytes). */
  signMessage: (message: Uint8Array) => Promise<Uint8Array>;
  /** Sign and send a transaction (returns the tx signature string). */
  sendTransaction: (transaction: Uint8Array) => Promise<string>;
}

// ── Hook ────────────────────────────────────────────────────────────────────

export function useWallet(): UseWalletReturn {
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [publicKey, setPublicKey] = useState<Uint8Array | null>(null);
  const [address, setAddress] = useState<string | null>(null);
  const [balance, setBalance] = useState<number | null>(null);
  const [walletName, setWalletName] = useState<string | null>(null);
  const [walletIcon, setWalletIcon] = useState<string | null>(null);

  // Hold a reference to the raw Wallet Standard wallet.
  const walletRef = useRef<DetectedWallet["raw"] | null>(null);

  // ── Connect ─────────────────────────────────────────────────────────

  const connect = useCallback(async (wallet: DetectedWallet) => {
    setConnecting(true);

    try {
      const connectFeature = wallet.raw.features["standard:connect"] as any;
      if (!connectFeature?.connect) {
        throw new Error(`Wallet "${wallet.name}" does not support connect.`);
      }

      const result = await connectFeature.connect();
      const accounts = result.accounts;

      if (!accounts || accounts.length === 0) {
        throw new Error("No accounts returned from wallet.");
      }

      const account = accounts[0];
      walletRef.current = wallet.raw;
      setPublicKey(account.publicKey);
      setAddress(account.address);
      setWalletName(wallet.name);
      setWalletIcon(typeof wallet.icon === "string" ? wallet.icon : null);
      setConnected(true);
    } catch (error) {
      walletRef.current = null;
      setConnected(false);
      throw error;
    } finally {
      setConnecting(false);
    }
  }, []);

  // ── Disconnect ──────────────────────────────────────────────────────

  const disconnect = useCallback(async () => {
    if (!walletRef.current) return;

    try {
      const disconnectFeature = walletRef.current.features[
        "standard:disconnect"
      ] as any;
      if (disconnectFeature?.disconnect) {
        await disconnectFeature.disconnect();
      }
    } finally {
      walletRef.current = null;
      setPublicKey(null);
      setAddress(null);
      setBalance(null);
      setWalletName(null);
      setWalletIcon(null);
      setConnected(false);
    }
  }, []);

  // ── Sign Transaction ────────────────────────────────────────────────

  const signTransaction = useCallback(
    async (transaction: Uint8Array): Promise<Uint8Array> => {
      if (!walletRef.current) {
        throw new Error("Wallet is not connected.");
      }

      const feature = walletRef.current.features[
        "solana:signTransaction"
      ] as any;
      if (!feature?.signTransaction) {
        throw new Error("Connected wallet does not support signTransaction.");
      }

      const [result] = await feature.signTransaction({
        transaction,
        account: walletRef.current.accounts[0],
      });

      return result.signedTransaction;
    },
    [],
  );

  // ── Sign Message ────────────────────────────────────────────────────

  const signMessage = useCallback(
    async (message: Uint8Array): Promise<Uint8Array> => {
      if (!walletRef.current) {
        throw new Error("Wallet is not connected.");
      }

      const feature = walletRef.current.features[
        "solana:signMessage"
      ] as any;
      if (!feature?.signMessage) {
        throw new Error("Connected wallet does not support signMessage.");
      }

      const [result] = await feature.signMessage({
        message,
        account: walletRef.current.accounts[0],
      });

      return result.signature;
    },
    [],
  );

  // ── Send Transaction ────────────────────────────────────────────────

  const sendTransaction = useCallback(
    async (transaction: Uint8Array): Promise<string> => {
      if (!walletRef.current) {
        throw new Error("Wallet is not connected.");
      }

      const feature = walletRef.current.features[
        "solana:signAndSendTransaction"
      ] as any;
      if (!feature?.signAndSendTransaction) {
        throw new Error(
          "Connected wallet does not support signAndSendTransaction.",
        );
      }

      const [result] = await feature.signAndSendTransaction({
        transaction,
        account: walletRef.current.accounts[0],
      });

      // Convert signature bytes to base58.
      const bs58 = await import("bs58");
      return bs58.default.encode(result.signature);
    },
    [],
  );

  // ── Balance Polling ─────────────────────────────────────────────────

  useEffect(() => {
    if (!connected || !address) {
      setBalance(null);
      return;
    }

    let cancelled = false;

    const fetchBalance = async () => {
      try {
        // Use the SolClone RPC to fetch balance.
        const response = await fetch("https://rpc.solclone.io", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            jsonrpc: "2.0",
            id: 1,
            method: "getBalance",
            params: [address],
          }),
        });

        const data = await response.json();
        if (!cancelled && data.result?.value !== undefined) {
          setBalance(data.result.value);
        }
      } catch {
        // Silently fail balance fetch — non-critical.
      }
    };

    fetchBalance();
    const interval = setInterval(fetchBalance, 30_000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [connected, address]);

  return {
    connected,
    connecting,
    publicKey,
    address,
    balance,
    walletName,
    walletIcon,
    connect,
    disconnect,
    signTransaction,
    signMessage,
    sendTransaction,
  };
}
