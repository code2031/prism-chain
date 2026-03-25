/**
 * @solclone/wallet-connect
 *
 * WalletConnect v2 protocol support for SolClone.
 * Enables mobile wallets to connect to web DApps via QR codes.
 */

// WalletConnect client
export { SolCloneWalletConnectClient } from "./client";
export type {
  WalletConnectConfig,
  WalletConnectSession,
  WalletConnectEventHandler,
} from "./client";

// QR code generation
export { WalletConnectQRModal } from "./qr-modal";
export type { QRCodeOptions, QRCodeResult } from "./qr-modal";

// Chain definitions
export {
  SOLCLONE_MAINNET,
  SOLCLONE_TESTNET,
  SOLCLONE_DEVNET,
  SOLANA_MAINNET,
  SOLANA_TESTNET,
  SOLANA_DEVNET,
  ALL_CHAINS,
  CHAIN_MAP,
  getChain,
  getNamespaces,
} from "./chains";
export type { ChainDefinition } from "./chains";
