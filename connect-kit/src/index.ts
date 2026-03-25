/**
 * @solclone/connect-kit
 *
 * React component library for adding wallet connections to SolClone DApps.
 * Drop-in components and hooks for wallet connect, disconnect, sign, and send.
 */

// Provider
export { SolCloneProvider, useSolCloneContext } from "./SolCloneProvider";
export type {
  SolCloneProviderProps,
  SolCloneContextValue,
  SolCloneNetwork,
} from "./SolCloneProvider";

// Components
export { ConnectButton } from "./ConnectButton";
export type { ConnectButtonProps } from "./ConnectButton";

export { WalletModal } from "./WalletModal";
export type { WalletModalProps } from "./WalletModal";

// Hooks
export { useWallet } from "./useWallet";
export type { UseWalletReturn } from "./useWallet";

export { useSolClone } from "./useSolClone";
export type {
  UseSolCloneReturn,
  SolCloneNetwork as SolCloneNetworkType,
  TokenAccount,
} from "./useSolClone";
