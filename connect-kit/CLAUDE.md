# CLAUDE.md — connect-kit

Guidance for Claude Code when working in this package.

## Overview

SolClone Connect Kit. Drop-in React component library for DApp wallet connection. Provides `SolCloneProvider`, `ConnectButton`, `WalletModal`, `useWallet`, and `useSolClone`. Composes `@solclone/wallet-standard` and `@solclone/wallet-connect` under a unified React context. TypeScript + React.

## Build & Run

```bash
cd connect-kit
npm install
npm run build        # tsc + bundle (outputs to dist/)
npm run dev          # Storybook dev server for component development
npm run lint         # ESLint
npm test             # Jest/Vitest + React Testing Library
```

Output goes to `dist/`. Package entry point is `dist/index.js` with types at `dist/index.d.ts`.

## Key Files

- `src/provider.tsx` — `SolCloneProvider` React context provider; manages wallet state, connection, network
- `src/components/ConnectButton.tsx` — Connect/disconnect button; shows truncated address when connected
- `src/components/WalletModal.tsx` — Modal overlay listing detected wallets with icons and install links
- `src/hooks/useWallet.ts` — Hook exposing `publicKey`, `connected`, `signTransaction`, `signMessage`, `disconnect`
- `src/hooks/useSolClone.ts` — Hook exposing `connection`, `balance`, `network`, `cluster`
- `src/theme.ts` — Light/dark theme tokens and CSS variable generation
- `src/detect.ts` — Aggregates wallets from wallet-standard and wallet-connect

## Architecture

`SolCloneProvider` is the root. It initializes wallet detection (via `@solclone/wallet-standard`) and WalletConnect (via `@solclone/wallet-connect`), manages active wallet state in React context, and exposes it through `useWallet` and `useSolClone`. The UI components (`ConnectButton`, `WalletModal`) consume these hooks internally.

## Dependencies

- `@solclone/wallet-standard` — Wallet Standard detection and registration
- `@solclone/wallet-connect` — WalletConnect v2 mobile pairing
- `@solclone/web3js-sdk` — Connection, transaction types
- `react`, `react-dom` — Peer dependencies (>=18)
