# SolClone Connect Kit

Drop-in React components for connecting wallets to SolClone DApps. Add full wallet support to any React application in three lines of code.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Features

- `<ConnectButton />` — One-click wallet connect/disconnect button
- `<WalletModal />` — Wallet selection modal with icons and install links
- `<SolCloneProvider />` — Context provider wrapping your app
- `useWallet()` — Hook for wallet state, signing, and sending transactions
- `useSolClone()` — Hook for RPC connection, balance, and network info
- Auto-detects Phantom, Solflare, Backpack, and all Wallet Standard wallets
- WalletConnect QR pairing for mobile wallets
- Responsive, themeable, accessible (WAI-ARIA)

## Installation

```bash
npm install @solclone/connect-kit
```

## Quick Start

```tsx
import { SolCloneProvider, ConnectButton } from "@solclone/connect-kit";

function App() {
  return (
    <SolCloneProvider network="devnet">
      <ConnectButton />
    </SolCloneProvider>
  );
}
```

That is it. Three lines to add wallet support to any DApp.

## Using the Hooks

```tsx
import { useWallet, useSolClone } from "@solclone/connect-kit";

function SendButton() {
  const { publicKey, signTransaction } = useWallet();
  const { connection, balance } = useSolClone();

  const handleSend = async () => {
    const tx = /* build transaction */;
    const signed = await signTransaction(tx);
    await connection.sendTransaction(signed);
  };

  return <button onClick={handleSend}>Send ({balance} SOL)</button>;
}
```

## Build

```bash
npm install
npm run build     # Compile TypeScript + bundle React components
npm run dev       # Start Storybook for component development
npm run lint      # Run ESLint
npm test          # Run test suite
```

## Theming

Supports `"light"`, `"dark"`, and `"auto"` (follows system preference):

```tsx
<SolCloneProvider network="devnet" theme="dark">
```

## Architecture

Connect Kit composes `@solclone/wallet-standard` (browser extension wallets) and `@solclone/wallet-connect` (mobile QR pairing) behind a single React context. DApp developers interact only with the hooks and components; wallet discovery, connection, and signing are handled internally.

## License

Apache-2.0. See [LICENSE](./LICENSE).
