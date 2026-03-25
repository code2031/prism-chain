# SolClone WalletConnect

WalletConnect v2 integration for the SolClone blockchain. Enables mobile wallet pairing via QR codes, allowing users to sign transactions on their phone while interacting with DApps on desktop.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Features

- WalletConnect v2 Sign Client for SolClone
- QR code generation for mobile wallet pairing
- SolClone chain ID definitions (devnet, testnet, mainnet)
- Session management (connect, disconnect, reconnect)
- Sign transaction and sign message over relay
- Deep link support for mobile-to-mobile flows

## Installation

```bash
npm install @solclone/wallet-connect
```

## Quick Start

```typescript
import { SolCloneWalletConnectClient, WalletConnectQRModal } from "@solclone/wallet-connect";

const client = new SolCloneWalletConnectClient({
  projectId: "your-walletconnect-project-id",
});
await client.init();

const { uri, approval } = await client.connect();
new WalletConnectQRModal().generateQR(uri);

const session = await approval;
const { signature } = await client.signTransaction(base64EncodedTx);
```

## Build

```bash
npm install
npm run build     # Compile TypeScript to dist/
npm run lint      # Run ESLint
npm test          # Run test suite
```

## Key Files

| File | Description |
|------|-------------|
| `src/client.ts` | WalletConnect `SignClient` wrapper with SolClone-specific methods |
| `src/qr-modal.ts` | QR code generation and display modal for pairing URIs |
| `src/chains.ts` | SolClone chain ID definitions (`solclone:devnet`, `solclone:testnet`, `solclone:mainnet`) |

## Architecture

```
Desktop DApp  -->  WalletConnect Relay  -->  Mobile Wallet
     |                    ^
     v                    |
  client.ts          qr-modal.ts
  (SignClient)       (pairing URI)
```

The client wraps the WalletConnect v2 `SignClient` and translates SolClone transactions into the relay protocol. On connect, `qr-modal.ts` renders a scannable QR code. Once paired, all sign requests route through the encrypted relay to the user's mobile wallet. Sessions persist across page reloads.

## Supported Chains

`solclone:devnet` (development), `solclone:testnet` (staging), `solclone:mainnet` (production)

## License

Apache-2.0. See [LICENSE](./LICENSE).
