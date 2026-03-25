/**
 * @solclone/wallet-standard
 *
 * Solana Wallet Standard implementation for SolClone.
 * Enables bidirectional wallet interoperability:
 *   - Third-party wallets (Phantom, Solflare, Backpack, ...) can connect to SolClone DApps.
 *   - SolClone's own wallet can connect to external Solana DApps.
 */

// Wallet implementation
export { SolCloneWallet, SolCloneWalletAccount } from "./wallet";
export type { KeypairProvider, TransactionSender } from "./wallet";

// Registration
export {
  registerSolCloneWallet,
  registerSolCloneWalletGlobally,
} from "./register";

// Feature constants
export {
  SolCloneConnect,
  SolCloneDisconnect,
  SolCloneEvents,
  SolCloneSignTransaction,
  SolCloneSignAndSendTransaction,
  SolCloneSignMessage,
  SOLCLONE_FEATURES,
  ALL_FEATURES,
} from "./features";
export type { SolCloneFeature } from "./features";

// Wallet detection
export { detectWallets, onWalletRegistered } from "./detect";
export type { DetectedWallet, DetectWalletsOptions } from "./detect";

// Wallet adapter bridge
export {
  SolCloneWalletAdapter,
  createSolCloneAdapter,
} from "./adapter";
export type { WalletAdapterInterface } from "./adapter";
