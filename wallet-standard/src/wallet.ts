/**
 * SolCloneWallet — Wallet Standard implementation for SolClone.
 *
 * This class implements the Wallet Standard interface so that any DApp
 * using the standard wallet discovery mechanism can detect, connect to,
 * and interact with the SolClone wallet.
 */

import type {
  Wallet,
  WalletAccount,
  WalletIcon,
} from "@wallet-standard/base";
import type {
  StandardConnectFeature,
  StandardConnectMethod,
  StandardDisconnectFeature,
  StandardDisconnectMethod,
  StandardEventsFeature,
  StandardEventsOnMethod,
  StandardEventsListeners,
} from "@wallet-standard/features";
import type {
  SolanaSignTransactionFeature,
  SolanaSignTransactionMethod,
  SolanaSignAndSendTransactionFeature,
  SolanaSignAndSendTransactionMethod,
  SolanaSignMessageFeature,
  SolanaSignMessageMethod,
} from "@solana/wallet-standard-features";
import {
  SolCloneConnect,
  SolCloneDisconnect,
  SolCloneEvents,
  SolCloneSignTransaction,
  SolCloneSignAndSendTransaction,
  SolCloneSignMessage,
} from "./features";

// ── Constants ───────────────────────────────────────────────────────────────

const SOLCLONE_WALLET_NAME = "SolClone" as const;

/**
 * SolClone logo encoded as a data URI (purple gradient S emblem).
 * Wallets must provide an icon in data-URI format per the Wallet Standard spec.
 */
const SOLCLONE_ICON: WalletIcon =
  "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTI4IiBoZWlnaHQ9IjEyOCIgdmlld0JveD0iMCAwIDEyOCAxMjgiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PGRlZnM+PGxpbmVhckdyYWRpZW50IGlkPSJnIiB4MT0iMCUiIHkxPSIwJSIgeDI9IjEwMCUiIHkyPSIxMDAlIj48c3RvcCBvZmZzZXQ9IjAlIiBzdG9wLWNvbG9yPSIjOUIzN0ZGIi8+PHN0b3Agb2Zmc2V0PSIxMDAlIiBzdG9wLWNvbG9yPSIjNkEwREFEIi8+PC9saW5lYXJHcmFkaWVudD48L2RlZnM+PHJlY3Qgd2lkdGg9IjEyOCIgaGVpZ2h0PSIxMjgiIHJ4PSIyNCIgZmlsbD0idXJsKCNnKSIvPjx0ZXh0IHg9IjY0IiB5PSI4OCIgZm9udC1mYW1pbHk9IkFyaWFsLHNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iNzIiIGZvbnQtd2VpZ2h0PSJib2xkIiBmaWxsPSJ3aGl0ZSIgdGV4dC1hbmNob3I9Im1pZGRsZSI+Uzwv dGV4dD48L3N2Zz4=" as WalletIcon;

/** All chains the SolClone wallet can operate on. */
const SOLCLONE_CHAINS = [
  "solana:mainnet",
  "solana:testnet",
  "solana:devnet",
  "solclone:mainnet",
  "solclone:testnet",
  "solclone:devnet",
] as const;

type SolCloneChain = (typeof SOLCLONE_CHAINS)[number];

// ── SolClone Wallet Account ─────────────────────────────────────────────────

/** Represents a single account exposed by the SolClone wallet. */
export class SolCloneWalletAccount implements WalletAccount {
  readonly address: string;
  readonly publicKey: Uint8Array;
  readonly chains: readonly string[];
  readonly features: readonly string[];
  readonly label?: string;
  readonly icon?: WalletIcon;

  constructor(params: {
    address: string;
    publicKey: Uint8Array;
    label?: string;
  }) {
    this.address = params.address;
    this.publicKey = params.publicKey;
    this.chains = [...SOLCLONE_CHAINS];
    this.features = [
      SolCloneSignTransaction,
      SolCloneSignAndSendTransaction,
      SolCloneSignMessage,
    ];
    this.label = params.label;
    this.icon = SOLCLONE_ICON;
  }
}

// ── Event Emitter ───────────────────────────────────────────────────────────

type EventType = keyof StandardEventsListeners;
type Listener = (...args: any[]) => void;

class WalletEventEmitter {
  private listeners = new Map<EventType, Set<Listener>>();

  on(event: EventType, listener: Listener): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(listener);
    return () => {
      this.listeners.get(event)?.delete(listener);
    };
  }

  emit(event: EventType, ...args: any[]): void {
    const handlers = this.listeners.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(...args);
        } catch {
          // Swallow listener errors to prevent cascading failures.
        }
      }
    }
  }
}

// ── Keypair Storage Interface ───────────────────────────────────────────────

/**
 * Abstraction for reading the user's keypair. Concrete implementations might
 * read from browser localStorage, a mobile keychain, or an in-memory store.
 */
export interface KeypairProvider {
  /** Returns the 64-byte secret key, or null if the wallet is locked. */
  getSecretKey(): Promise<Uint8Array | null>;
  /** Returns the 32-byte public key. */
  getPublicKey(): Promise<Uint8Array | null>;
  /** Returns the base58-encoded public key (address). */
  getAddress(): Promise<string | null>;
}

// ── RPC Sender Interface ────────────────────────────────────────────────────

/** Abstraction for broadcasting signed transactions to the network. */
export interface TransactionSender {
  sendRawTransaction(rawTransaction: Uint8Array): Promise<string>;
}

// ── SolClone Wallet ─────────────────────────────────────────────────────────

export class SolCloneWallet
  implements Wallet
{
  readonly #keypairProvider: KeypairProvider;
  readonly #transactionSender: TransactionSender;
  readonly #emitter = new WalletEventEmitter();

  #accounts: SolCloneWalletAccount[] = [];
  #connected = false;

  // ── Wallet Standard identity properties ─────────────────────────────────

  readonly version = "1.0.0" as const;
  readonly name = SOLCLONE_WALLET_NAME;
  readonly icon = SOLCLONE_ICON;
  readonly chains = SOLCLONE_CHAINS;

  get accounts(): readonly WalletAccount[] {
    return this.#accounts;
  }

  // ── Features map (Wallet Standard discovery) ────────────────────────────

  get features(): StandardConnectFeature &
    StandardDisconnectFeature &
    StandardEventsFeature &
    SolanaSignTransactionFeature &
    SolanaSignAndSendTransactionFeature &
    SolanaSignMessageFeature {
    return {
      [SolCloneConnect]: {
        version: "1.0.0",
        connect: this.#connect,
      },
      [SolCloneDisconnect]: {
        version: "1.0.0",
        disconnect: this.#disconnect,
      },
      [SolCloneEvents]: {
        version: "1.0.0",
        on: this.#on,
      },
      [SolCloneSignTransaction]: {
        version: "1.0.0",
        supportedTransactionVersions: ["legacy", 0],
        signTransaction: this.#signTransaction,
      },
      [SolCloneSignAndSendTransaction]: {
        version: "1.0.0",
        supportedTransactionVersions: ["legacy", 0],
        signAndSendTransaction: this.#signAndSendTransaction,
      },
      [SolCloneSignMessage]: {
        version: "1.0.0",
        signMessage: this.#signMessage,
      },
    };
  }

  // ── Constructor ─────────────────────────────────────────────────────────

  constructor(
    keypairProvider: KeypairProvider,
    transactionSender: TransactionSender,
  ) {
    this.#keypairProvider = keypairProvider;
    this.#transactionSender = transactionSender;
  }

  // ── Connect / Disconnect ────────────────────────────────────────────────

  #connect: StandardConnectMethod = async (input) => {
    if (this.#connected && this.#accounts.length > 0) {
      return { accounts: this.accounts };
    }

    const publicKey = await this.#keypairProvider.getPublicKey();
    const address = await this.#keypairProvider.getAddress();

    if (!publicKey || !address) {
      throw new Error("SolClone wallet is locked or has no keypair.");
    }

    const account = new SolCloneWalletAccount({
      address,
      publicKey,
    });

    this.#accounts = [account];
    this.#connected = true;
    this.#emitter.emit("change", { accounts: this.accounts });

    return { accounts: this.accounts };
  };

  #disconnect: StandardDisconnectMethod = async () => {
    this.#accounts = [];
    this.#connected = false;
    this.#emitter.emit("change", { accounts: this.accounts });
  };

  // ── Events ──────────────────────────────────────────────────────────────

  #on: StandardEventsOnMethod = (event, listener) => {
    return this.#emitter.on(event, listener);
  };

  // ── Sign Transaction ────────────────────────────────────────────────────

  #signTransaction: SolanaSignTransactionMethod = async (...inputs) => {
    const secretKey = await this.#keypairProvider.getSecretKey();
    if (!secretKey) {
      throw new Error("SolClone wallet is locked.");
    }

    const nacl = await import("tweetnacl");
    const outputs: { signedTransaction: Uint8Array }[] = [];

    for (const input of inputs) {
      const transaction =
        input.transaction instanceof Uint8Array
          ? input.transaction
          : new Uint8Array(input.transaction);

      const signature = nacl.sign.detached(transaction, secretKey);

      // Build a signed transaction by prepending the signature.
      // Real-world implementations would properly serialize the
      // Solana transaction structure; this is a simplified version.
      const signedTransaction = new Uint8Array(
        signature.length + transaction.length,
      );
      signedTransaction.set(signature, 0);
      signedTransaction.set(transaction, signature.length);

      outputs.push({ signedTransaction });
    }

    return outputs;
  };

  // ── Sign and Send Transaction ───────────────────────────────────────────

  #signAndSendTransaction: SolanaSignAndSendTransactionMethod = async (
    ...inputs
  ) => {
    const secretKey = await this.#keypairProvider.getSecretKey();
    if (!secretKey) {
      throw new Error("SolClone wallet is locked.");
    }

    const nacl = await import("tweetnacl");
    const outputs: { signature: Uint8Array }[] = [];

    for (const input of inputs) {
      const transaction =
        input.transaction instanceof Uint8Array
          ? input.transaction
          : new Uint8Array(input.transaction);

      const sig = nacl.sign.detached(transaction, secretKey);

      // Build signed transaction and send it.
      const signedTransaction = new Uint8Array(
        sig.length + transaction.length,
      );
      signedTransaction.set(sig, 0);
      signedTransaction.set(transaction, sig.length);

      await this.#transactionSender.sendRawTransaction(signedTransaction);
      outputs.push({ signature: sig });
    }

    return outputs;
  };

  // ── Sign Message ────────────────────────────────────────────────────────

  #signMessage: SolanaSignMessageMethod = async (...inputs) => {
    const secretKey = await this.#keypairProvider.getSecretKey();
    if (!secretKey) {
      throw new Error("SolClone wallet is locked.");
    }

    const nacl = await import("tweetnacl");
    const outputs: { signedMessage: Uint8Array; signature: Uint8Array }[] = [];

    for (const input of inputs) {
      const message =
        input.message instanceof Uint8Array
          ? input.message
          : new Uint8Array(input.message);

      const signature = nacl.sign.detached(message, secretKey);

      outputs.push({
        signedMessage: message,
        signature,
      });
    }

    return outputs;
  };
}
