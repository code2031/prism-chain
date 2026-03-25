/**
 * SolCloneWalletConnectClient — WalletConnect v2 integration for SolClone.
 *
 * Allows mobile wallets to connect to web DApps via QR codes using
 * the WalletConnect relay protocol. Handles session lifecycle,
 * transaction signing, and message signing.
 */

import type SignClient from "@walletconnect/sign-client";
import type {
  SessionTypes,
  SignClientTypes,
  PairingTypes,
} from "@walletconnect/types";
import { getNamespaces, type ChainDefinition, SOLCLONE_MAINNET } from "./chains";

// ── Types ───────────────────────────────────────────────────────────────────

export interface WalletConnectConfig {
  /** WalletConnect Cloud project ID (get one at cloud.walletconnect.com). */
  projectId: string;
  /** Optional metadata for the DApp. */
  metadata?: {
    name: string;
    description: string;
    url: string;
    icons: string[];
  };
  /** Chain to use by default. Defaults to solclone:mainnet. */
  defaultChain?: ChainDefinition;
}

export interface WalletConnectSession {
  topic: string;
  peerName: string;
  peerUrl: string;
  peerIcon: string | undefined;
  accounts: string[];
  chains: string[];
  expiry: number;
}

export type WalletConnectEventHandler = (event: {
  type: string;
  data: any;
}) => void;

// ── Client ──────────────────────────────────────────────────────────────────

export class SolCloneWalletConnectClient {
  #signClient: SignClient | null = null;
  #session: SessionTypes.Struct | null = null;
  #config: WalletConnectConfig;
  #eventHandlers = new Map<string, Set<WalletConnectEventHandler>>();

  constructor(config: WalletConnectConfig) {
    this.#config = config;
  }

  // ── Lifecycle ─────────────────────────────────────────────────────────

  /**
   * Initialize the WalletConnect SignClient. Must be called before
   * any other method.
   */
  async init(): Promise<void> {
    const { SignClient: SC } = await import("@walletconnect/sign-client");

    this.#signClient = await (SC as any).init({
      projectId: this.#config.projectId,
      metadata: this.#config.metadata ?? {
        name: "SolClone DApp",
        description: "A decentralized application on SolClone",
        url: "https://solclone.io",
        icons: ["https://solclone.io/icon.png"],
      },
    });

    this.#setupEventListeners();

    // Restore existing session if available.
    const sessions = this.#signClient.session.getAll();
    if (sessions.length > 0) {
      this.#session = sessions[sessions.length - 1];
      this.#emit("session_restored", this.#formatSession(this.#session));
    }
  }

  /**
   * Create a pairing and return the WalletConnect URI for QR code display.
   * The user scans this QR code with their mobile wallet.
   *
   * @returns The WalletConnect URI string.
   */
  async connect(): Promise<{ uri: string; approval: Promise<WalletConnectSession> }> {
    this.#requireClient();

    const namespaces = getNamespaces();

    const { uri, approval } = await this.#signClient!.connect({
      requiredNamespaces: {
        solclone: {
          chains: namespaces.solclone.chains,
          methods: namespaces.solclone.methods,
          events: namespaces.solclone.events,
        },
      },
      optionalNamespaces: {
        solana: {
          chains: namespaces.solana.chains,
          methods: namespaces.solana.methods,
          events: namespaces.solana.events,
        },
      },
    });

    if (!uri) {
      throw new Error("Failed to generate WalletConnect URI.");
    }

    const sessionPromise = approval().then((session) => {
      this.#session = session;
      const formatted = this.#formatSession(session);
      this.#emit("session_created", formatted);
      return formatted;
    });

    return { uri, approval: sessionPromise };
  }

  /**
   * Approve an incoming session proposal (wallet-side).
   * Called when the SolClone wallet receives a `session_proposal` event.
   */
  async approve(
    proposalId: number,
    accounts: string[],
    chains: string[],
  ): Promise<WalletConnectSession> {
    this.#requireClient();

    const namespaces: Record<string, SessionTypes.Namespace> = {};

    // Group accounts by namespace.
    const solcloneAccounts = accounts.filter((a) => a.startsWith("solclone:"));
    const solanaAccounts = accounts.filter((a) => a.startsWith("solana:"));

    if (solcloneAccounts.length > 0) {
      namespaces.solclone = {
        accounts: solcloneAccounts,
        chains: chains.filter((c) => c.startsWith("solclone:")),
        methods: [
          "solana_signTransaction",
          "solana_signAndSendTransaction",
          "solana_signMessage",
        ],
        events: ["accountsChanged", "chainChanged"],
      };
    }

    if (solanaAccounts.length > 0) {
      namespaces.solana = {
        accounts: solanaAccounts,
        chains: chains.filter((c) => c.startsWith("solana:")),
        methods: [
          "solana_signTransaction",
          "solana_signAndSendTransaction",
          "solana_signMessage",
        ],
        events: ["accountsChanged", "chainChanged"],
      };
    }

    const session = await this.#signClient!.approve({
      id: proposalId,
      namespaces,
    });

    this.#session = session.acknowledged
      ? await session.acknowledged()
      : (session as any);

    const formatted = this.#formatSession(this.#session!);
    this.#emit("session_approved", formatted);
    return formatted;
  }

  /**
   * Reject an incoming session proposal.
   */
  async reject(proposalId: number, reason?: string): Promise<void> {
    this.#requireClient();

    await this.#signClient!.reject({
      id: proposalId,
      reason: {
        code: 5000,
        message: reason ?? "User rejected the session.",
      },
    });
  }

  // ── Signing ───────────────────────────────────────────────────────────

  /**
   * Request the connected wallet to sign a transaction.
   *
   * @param transaction - Base64-encoded serialized transaction.
   * @returns The signature as a base58 string.
   */
  async signTransaction(transaction: string): Promise<{ signature: string }> {
    this.#requireSession();

    const chain =
      this.#config.defaultChain?.id ?? SOLCLONE_MAINNET.id;

    const result = await this.#signClient!.request({
      topic: this.#session!.topic,
      chainId: chain,
      request: {
        method: "solana_signTransaction",
        params: { transaction },
      },
    });

    return result as { signature: string };
  }

  /**
   * Request the connected wallet to sign and send a transaction.
   *
   * @param transaction - Base64-encoded serialized transaction.
   * @returns The transaction signature.
   */
  async signAndSendTransaction(
    transaction: string,
  ): Promise<{ signature: string }> {
    this.#requireSession();

    const chain =
      this.#config.defaultChain?.id ?? SOLCLONE_MAINNET.id;

    const result = await this.#signClient!.request({
      topic: this.#session!.topic,
      chainId: chain,
      request: {
        method: "solana_signAndSendTransaction",
        params: { transaction },
      },
    });

    return result as { signature: string };
  }

  /**
   * Request the connected wallet to sign an arbitrary message.
   *
   * @param message - Base64-encoded message bytes.
   * @returns The signature as a base58 string.
   */
  async signMessage(message: string): Promise<{ signature: string }> {
    this.#requireSession();

    const chain =
      this.#config.defaultChain?.id ?? SOLCLONE_MAINNET.id;

    const result = await this.#signClient!.request({
      topic: this.#session!.topic,
      chainId: chain,
      request: {
        method: "solana_signMessage",
        params: { message },
      },
    });

    return result as { signature: string };
  }

  // ── Disconnect ────────────────────────────────────────────────────────

  /**
   * Disconnect the current session.
   */
  async disconnect(): Promise<void> {
    if (!this.#signClient || !this.#session) return;

    try {
      await this.#signClient.disconnect({
        topic: this.#session.topic,
        reason: {
          code: 6000,
          message: "User disconnected.",
        },
      });
    } finally {
      const topic = this.#session.topic;
      this.#session = null;
      this.#emit("session_deleted", { topic });
    }
  }

  // ── Session Info ──────────────────────────────────────────────────────

  /** Returns the current session, or null. */
  getSession(): WalletConnectSession | null {
    return this.#session ? this.#formatSession(this.#session) : null;
  }

  /** Whether a session is currently active. */
  get isConnected(): boolean {
    return this.#session !== null;
  }

  /** All active pairings. */
  getPairings(): PairingTypes.Struct[] {
    return this.#signClient?.pairing.getAll({ active: true }) ?? [];
  }

  // ── Event Handling ────────────────────────────────────────────────────

  /**
   * Register an event handler.
   * Supported events: session_proposal, session_request, session_delete,
   * session_created, session_approved, session_restored.
   */
  on(event: string, handler: WalletConnectEventHandler): () => void {
    if (!this.#eventHandlers.has(event)) {
      this.#eventHandlers.set(event, new Set());
    }
    this.#eventHandlers.get(event)!.add(handler);
    return () => {
      this.#eventHandlers.get(event)?.delete(handler);
    };
  }

  // ── Private ───────────────────────────────────────────────────────────

  #setupEventListeners(): void {
    if (!this.#signClient) return;

    this.#signClient.on("session_proposal", (event) => {
      this.#emit("session_proposal", event);
    });

    this.#signClient.on("session_request", async (event) => {
      this.#emit("session_request", event);
    });

    this.#signClient.on("session_delete", (event) => {
      if (this.#session?.topic === event.topic) {
        this.#session = null;
      }
      this.#emit("session_delete", event);
    });
  }

  #emit(event: string, data: any): void {
    const handlers = this.#eventHandlers.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler({ type: event, data });
        } catch {
          // Swallow listener errors.
        }
      }
    }
  }

  #formatSession(session: SessionTypes.Struct): WalletConnectSession {
    const peer = session.peer?.metadata;
    const allAccounts: string[] = [];
    const allChains: string[] = [];

    for (const ns of Object.values(session.namespaces)) {
      if (ns.accounts) allAccounts.push(...ns.accounts);
      if (ns.chains) allChains.push(...ns.chains);
    }

    return {
      topic: session.topic,
      peerName: peer?.name ?? "Unknown",
      peerUrl: peer?.url ?? "",
      peerIcon: peer?.icons?.[0],
      accounts: allAccounts,
      chains: allChains,
      expiry: session.expiry,
    };
  }

  #requireClient(): void {
    if (!this.#signClient) {
      throw new Error(
        "WalletConnect client not initialized. Call init() first.",
      );
    }
  }

  #requireSession(): void {
    this.#requireClient();
    if (!this.#session) {
      throw new Error(
        "No active WalletConnect session. Call connect() first.",
      );
    }
  }
}
