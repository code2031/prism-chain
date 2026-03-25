/**
 * SolClone Wallet Standard Feature Constants
 *
 * These feature identifiers are used to declare the capabilities
 * that the SolClone wallet supports. DApps query these features
 * to determine what operations they can request from the wallet.
 */

// ── Standard Solana Features ────────────────────────────────────────────────

/** Feature identifier for the connect capability. */
export const SolCloneConnect = "standard:connect" as const;

/** Feature identifier for the disconnect capability. */
export const SolCloneDisconnect = "standard:disconnect" as const;

/** Feature identifier for the events capability (state change notifications). */
export const SolCloneEvents = "standard:events" as const;

// ── Solana-Specific Features ────────────────────────────────────────────────

/** Feature identifier for signing a transaction without sending it. */
export const SolCloneSignTransaction = "solana:signTransaction" as const;

/** Feature identifier for signing and sending a transaction in one step. */
export const SolCloneSignAndSendTransaction = "solana:signAndSendTransaction" as const;

/** Feature identifier for signing an arbitrary message (off-chain). */
export const SolCloneSignMessage = "solana:signMessage" as const;

// ── Feature Type Map ────────────────────────────────────────────────────────

/**
 * All features supported by the SolClone wallet, grouped by category.
 */
export const SOLCLONE_FEATURES = {
  connect: SolCloneConnect,
  disconnect: SolCloneDisconnect,
  events: SolCloneEvents,
  signTransaction: SolCloneSignTransaction,
  signAndSendTransaction: SolCloneSignAndSendTransaction,
  signMessage: SolCloneSignMessage,
} as const;

/**
 * Array of all feature identifiers for easy iteration and registration.
 */
export const ALL_FEATURES = [
  SolCloneConnect,
  SolCloneDisconnect,
  SolCloneEvents,
  SolCloneSignTransaction,
  SolCloneSignAndSendTransaction,
  SolCloneSignMessage,
] as const;

/** Union type of all SolClone feature identifiers. */
export type SolCloneFeature = (typeof ALL_FEATURES)[number];
