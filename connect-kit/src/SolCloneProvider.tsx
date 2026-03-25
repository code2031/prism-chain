/**
 * SolCloneProvider — React context provider that wraps wallet-adapter
 * with SolClone networks preconfigured.
 *
 * Provides wallet state and methods to all child components via context.
 */

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useEffect,
  useRef,
  type ReactNode,
} from "react";
import { detectWallets, type DetectedWallet } from "@solclone/wallet-standard";

// ── Types ───────────────────────────────────────────────────────────────────

export type SolCloneNetwork = "devnet" | "testnet" | "mainnet";

export interface SolCloneContextValue {
  /** Whether a wallet is connected. */
  connected: boolean;
  /** Whether a connection attempt is in progress. */
  connecting: boolean;
  /** The connected wallet's public key (32 bytes), or null. */
  publicKey: Uint8Array | null;
  /** The connected wallet's base58 address, or null. */
  address: string | null;
  /** The currently selected network. */
  network: SolCloneNetwork;
  /** Info about the connected wallet. */
  wallet: { name: string; icon: string | undefined } | null;
  /** All detected Wallet Standard wallets. */
  wallets: DetectedWallet[];

  /** Connect to a specific wallet. */
  connect: (wallet: DetectedWallet) => Promise<void>;
  /** Disconnect the current wallet. */
  disconnect: () => Promise<void>;
  /** Sign a transaction. */
  signTransaction: (transaction: Uint8Array) => Promise<Uint8Array>;
  /** Sign an arbitrary message. */
  signMessage: (message: Uint8Array) => Promise<Uint8Array>;
  /** Sign and send a transaction, returning the signature string. */
  sendTransaction: (transaction: Uint8Array) => Promise<string>;
}

export interface SolCloneProviderProps {
  /** Network to connect to. Default: "devnet". */
  network?: SolCloneNetwork;
  /** Child components that will have access to the wallet context. */
  children: ReactNode;
}

// ── RPC URLs ────────────────────────────────────────────────────────────────

const RPC_URLS: Record<SolCloneNetwork, string> = {
  mainnet: "https://rpc.solclone.io",
  testnet: "https://testnet.rpc.solclone.io",
  devnet: "https://devnet.rpc.solclone.io",
};

// ── Context ─────────────────────────────────────────────────────────────────

const SolCloneContext = createContext<SolCloneContextValue | null>(null);

// ── Provider Component ──────────────────────────────────────────────────────

export function SolCloneProvider({
  network = "devnet",
  children,
}: SolCloneProviderProps) {
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [publicKey, setPublicKey] = useState<Uint8Array | null>(null);
  const [address, setAddress] = useState<string | null>(null);
  const [wallet, setWallet] = useState<{
    name: string;
    icon: string | undefined;
  } | null>(null);
  const [wallets, setWallets] = useState<DetectedWallet[]>([]);

  const rawWalletRef = useRef<DetectedWallet["raw"] | null>(null);

  // ── Detect Wallets ──────────────────────────────────────────────────

  useEffect(() => {
    // Detect wallets that support SolClone or Solana chains.
    const detected = detectWallets({
      chains: [
        `solclone:${network}`,
        `solana:${network === "mainnet" ? "mainnet" : network}`,
      ],
    });
    setWallets(detected);

    // Also detect wallets with no chain filter as fallback.
    if (detected.length === 0) {
      const all = detectWallets();
      setWallets(all);
    }
  }, [network]);

  // ── Connect ─────────────────────────────────────────────────────────

  const connect = useCallback(async (selectedWallet: DetectedWallet) => {
    setConnecting(true);

    try {
      const connectFeature = selectedWallet.raw.features[
        "standard:connect"
      ] as any;

      if (!connectFeature?.connect) {
        throw new Error(
          `Wallet "${selectedWallet.name}" does not support the connect feature.`,
        );
      }

      const result = await connectFeature.connect();
      const accounts = result.accounts;

      if (!accounts || accounts.length === 0) {
        throw new Error("No accounts returned from wallet.");
      }

      const account = accounts[0];
      rawWalletRef.current = selectedWallet.raw;

      setPublicKey(account.publicKey);
      setAddress(account.address);
      setWallet({
        name: selectedWallet.name,
        icon:
          typeof selectedWallet.icon === "string"
            ? selectedWallet.icon
            : undefined,
      });
      setConnected(true);
    } catch (error) {
      rawWalletRef.current = null;
      setConnected(false);
      throw error;
    } finally {
      setConnecting(false);
    }
  }, []);

  // ── Disconnect ──────────────────────────────────────────────────────

  const disconnect = useCallback(async () => {
    if (!rawWalletRef.current) return;

    try {
      const feature = rawWalletRef.current.features[
        "standard:disconnect"
      ] as any;
      if (feature?.disconnect) {
        await feature.disconnect();
      }
    } finally {
      rawWalletRef.current = null;
      setPublicKey(null);
      setAddress(null);
      setWallet(null);
      setConnected(false);
    }
  }, []);

  // ── Sign Transaction ────────────────────────────────────────────────

  const signTransaction = useCallback(
    async (transaction: Uint8Array): Promise<Uint8Array> => {
      if (!rawWalletRef.current) {
        throw new Error("No wallet connected.");
      }

      const feature = rawWalletRef.current.features[
        "solana:signTransaction"
      ] as any;

      if (!feature?.signTransaction) {
        throw new Error("Wallet does not support signTransaction.");
      }

      const [result] = await feature.signTransaction({
        transaction,
        account: rawWalletRef.current.accounts[0],
      });

      return result.signedTransaction;
    },
    [],
  );

  // ── Sign Message ────────────────────────────────────────────────────

  const signMessage = useCallback(
    async (message: Uint8Array): Promise<Uint8Array> => {
      if (!rawWalletRef.current) {
        throw new Error("No wallet connected.");
      }

      const feature = rawWalletRef.current.features[
        "solana:signMessage"
      ] as any;

      if (!feature?.signMessage) {
        throw new Error("Wallet does not support signMessage.");
      }

      const [result] = await feature.signMessage({
        message,
        account: rawWalletRef.current.accounts[0],
      });

      return result.signature;
    },
    [],
  );

  // ── Send Transaction ────────────────────────────────────────────────

  const sendTransaction = useCallback(
    async (transaction: Uint8Array): Promise<string> => {
      if (!rawWalletRef.current) {
        throw new Error("No wallet connected.");
      }

      const feature = rawWalletRef.current.features[
        "solana:signAndSendTransaction"
      ] as any;

      if (!feature?.signAndSendTransaction) {
        throw new Error("Wallet does not support signAndSendTransaction.");
      }

      const [result] = await feature.signAndSendTransaction({
        transaction,
        account: rawWalletRef.current.accounts[0],
      });

      const bs58 = await import("bs58");
      return bs58.default.encode(result.signature);
    },
    [],
  );

  // ── Context Value ───────────────────────────────────────────────────

  const value: SolCloneContextValue = {
    connected,
    connecting,
    publicKey,
    address,
    network,
    wallet,
    wallets,
    connect,
    disconnect,
    signTransaction,
    signMessage,
    sendTransaction,
  };

  return (
    <SolCloneContext.Provider value={value}>
      {children}
    </SolCloneContext.Provider>
  );
}

// ── Context Hook ────────────────────────────────────────────────────────────

/**
 * Access the SolClone wallet context. Must be used within a <SolCloneProvider>.
 */
export function useSolCloneContext(): SolCloneContextValue {
  const context = useContext(SolCloneContext);
  if (!context) {
    throw new Error(
      "useSolCloneContext must be used within a <SolCloneProvider>.",
    );
  }
  return context;
}
