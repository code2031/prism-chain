# @solclone/wallet-standard

Solana Wallet Standard implementation for SolClone. This package enables full bidirectional wallet interoperability:

- **Third-party wallets** (Phantom, Solflare, Backpack, etc.) can connect to SolClone DApps
- **SolClone's wallet** can connect to external Solana DApps

## Installation

```bash
npm install @solclone/wallet-standard
```

## Registering the SolClone Wallet

Call `registerSolCloneWallet()` once at startup (e.g., in your wallet extension's content script) so that DApps can discover SolClone through the Wallet Standard registry.

```ts
import { registerSolCloneWallet } from "@solclone/wallet-standard";

const keypairProvider = {
  async getSecretKey() { /* return 64-byte Uint8Array */ },
  async getPublicKey() { /* return 32-byte Uint8Array */ },
  async getAddress()   { /* return base58 string */      },
};

const transactionSender = {
  async sendRawTransaction(raw: Uint8Array) { /* send and return txid */ },
};

const unregister = registerSolCloneWallet(keypairProvider, transactionSender);
```

## Detecting Wallets in a DApp

Use `detectWallets()` to find every Wallet Standard wallet the user has installed. This is the foundation for building a wallet selection dialog.

```ts
import { detectWallets } from "@solclone/wallet-standard";

// All wallets
const wallets = detectWallets();

// Only wallets that support SolClone chains
const solcloneWallets = detectWallets({
  chains: ["solclone:mainnet", "solclone:devnet"],
});

// Only wallets that can sign transactions
const signers = detectWallets({
  requiredFeatures: ["solana:signTransaction"],
});

wallets.forEach(w => {
  console.log(w.name, w.icon, w.chains, w.features);
});
```

### Listening for New Wallets

Some wallets register lazily. Use `onWalletRegistered()` to react in real time:

```ts
import { onWalletRegistered } from "@solclone/wallet-standard";

const stop = onWalletRegistered((wallet) => {
  console.log("New wallet detected:", wallet.name);
});
```

## Using the Wallet Adapter Bridge

If your DApp uses `@solana/wallet-adapter-react`, the `SolCloneWalletAdapter` makes SolClone work seamlessly with the existing ecosystem:

```ts
import { SolCloneWalletAdapter } from "@solclone/wallet-standard";
import { WalletProvider } from "@solana/wallet-adapter-react";

const wallets = [new SolCloneWalletAdapter(solcloneWalletInstance)];

function App() {
  return (
    <WalletProvider wallets={wallets}>
      {/* your app */}
    </WalletProvider>
  );
}
```

Or use the convenience factory:

```ts
import { createSolCloneAdapter } from "@solclone/wallet-standard";

const adapter = createSolCloneAdapter(); // finds SolClone in the registry
```

## Supported Chains

| Chain               | Description          |
|---------------------|----------------------|
| `solana:mainnet`    | Solana Mainnet Beta  |
| `solana:testnet`    | Solana Testnet       |
| `solana:devnet`     | Solana Devnet        |
| `solclone:mainnet`  | SolClone Mainnet     |
| `solclone:testnet`  | SolClone Testnet     |
| `solclone:devnet`   | SolClone Devnet      |

## Supported Features

| Feature                         | Description                              |
|---------------------------------|------------------------------------------|
| `standard:connect`              | Connect the wallet                       |
| `standard:disconnect`           | Disconnect the wallet                    |
| `standard:events`               | Listen for state changes                 |
| `solana:signTransaction`        | Sign a transaction without sending       |
| `solana:signAndSendTransaction` | Sign and broadcast a transaction         |
| `solana:signMessage`            | Sign an arbitrary off-chain message      |

## Architecture

```
DApp  <-->  Wallet Standard Registry  <-->  SolCloneWallet
                                       |
                                       +-->  Phantom / Solflare / Backpack / ...
```

The Wallet Standard acts as a universal discovery layer. Any wallet that registers itself can be found by any DApp, and vice versa.
