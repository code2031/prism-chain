# SolClone Wallet Standard

Implements the [Wallet Standard](https://github.com/wallet-standard/wallet-standard) for the SolClone blockchain, enabling any compliant third-party wallet -- Phantom, Solflare, Backpack, and others -- to connect to SolClone DApps.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Features

- Full Wallet Standard compliance for SolClone chain
- Automatic wallet detection and registration
- Bridge layer for legacy `@solana/wallet-adapter` compatibility
- Support for Phantom, Solflare, Backpack, and any Wallet Standard wallet
- Sign transaction, sign message, and connect/disconnect flows

## Installation

```bash
npm install @solclone/wallet-standard
```

## Quick Start

```typescript
import { SolCloneWallet } from "@solclone/wallet-standard";
import { registerSolCloneWallet } from "@solclone/wallet-standard/register";

const wallet = new SolCloneWallet(keypairProvider, transactionSender);
registerSolCloneWallet(wallet);
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
| `src/wallet.ts` | `SolCloneWallet` class implementing the Wallet Standard interface |
| `src/adapter.ts` | Bridge between Wallet Standard and legacy wallet-adapter |
| `src/register.ts` | Wallet registration with the global wallet registry |
| `src/detect.ts` | Runtime detection of installed wallets |

## Architecture

```
DApp  -->  wallet-standard (detect + register)  -->  Wallet (Phantom, Solflare, etc.)
                  |
                  v
           wallet-adapter (legacy bridge)
```

The Wallet Standard acts as the universal interface layer. DApps built with `@solclone/connect-kit` or `@solclone/wallet-adapter` automatically discover any registered wallet through this package. The adapter bridge ensures backward compatibility with older DApps using the legacy wallet-adapter API.

## Supported Wallets

Phantom, Solflare, Backpack, and any wallet implementing the Wallet Standard are fully supported and auto-detected at runtime.

## Supported Chains

`solclone:mainnet`, `solclone:testnet`, `solclone:devnet`, `solana:mainnet`, `solana:testnet`, `solana:devnet`

## License

Apache-2.0. See [LICENSE](./LICENSE).
