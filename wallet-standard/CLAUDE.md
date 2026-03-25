# CLAUDE.md — wallet-standard

Guidance for Claude Code when working in this package.

## Overview

SolClone Wallet Standard implementation. Provides the interface layer that lets third-party wallets (Phantom, Solflare, Backpack) connect to SolClone DApps. TypeScript package, compiled with tsc.

## Build & Run

```bash
cd wallet-standard
npm install
npm run build        # Compile to dist/ (tsc)
npm run lint         # ESLint
npm test             # Jest/Vitest tests
```

Output goes to `dist/`. Package entry point is `dist/index.js` with types at `dist/index.d.ts`.

## Key Files

- `src/wallet.ts` — `SolCloneWallet` class, the core Wallet Standard implementation
- `src/adapter.ts` — Bridge that wraps Wallet Standard wallets in legacy wallet-adapter interface
- `src/register.ts` — Registers wallets with the global `window.__wallets__` registry
- `src/detect.ts` — Scans for installed browser-extension wallets at runtime
- `src/features.ts` — Feature descriptors: `solclone:signTransaction`, `solclone:signMessage`, `standard:connect`
- `src/types.ts` — Shared types (`SolCloneChain`, `WalletAccount`, `WalletFeature`)

## Architecture

The package sits between DApps and wallet browser extensions. `detect.ts` finds wallets, `register.ts` adds them to the registry, and `wallet.ts` exposes the standard interface. `adapter.ts` is only needed for DApps still using the legacy `@solclone/wallet-adapter` API.

## Dependencies

- `@wallet-standard/base` — Core Wallet Standard types
- `@solclone/web3js-sdk` — SolClone transaction and keypair types
