# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

SolClone is a complete Solana blockchain fork. This monorepo contains the validator (Rust), CLI wallet (TypeScript), Flutter GUI wallet (Dart), block explorer (Next.js), web3.js SDK, SPL programs, wallet adapter, DApp scaffold, and production operations tooling. The 7 forked upstream components each have their own split repo under `code2031/solclone-*`.

## Build & Run

The root `Makefile` is the primary entry point. All paths below assume repo root.

```bash
make validator         # Build validator (Rust, release, ~30 min)
make cli               # Build just the Solana CLI (faster)
make programs          # Build all SPL programs (Rust)
make token-program     # Build just SPL Token
make setup             # npm/pnpm install for all JS projects
make sdk               # Build web3.js SDK (rollup)
make status            # Show what's built and running
make clean             # Remove all build artifacts
```

### Local Testnet

```bash
make testnet-bg        # Start test validator in background (RPC :8899, WS :8900)
make stop-testnet      # Kill it
./scripts/airdrop.sh 100   # Fund yourself on localnet
```

### Per-Component Dev Servers

```bash
make explorer          # Block explorer on :3000 (pnpm dev)
make dapp              # DApp scaffold on :3000 (npm run dev)
make wallet            # Backpack wallet (yarn start)
```

### CLI Wallet

```bash
cd cli-wallet
npm install && npm run build   # Compiles to dist/ (tsc)
npx solclone keygen            # Generate Ed25519 keypair
npx solclone config use devnet # Switch network
npx solclone balance           # Check SOL balance
npx solclone transfer <addr> 1 # Send SOL
npx solclone token create-mint # Create SPL token
npx solclone stake delegate <validator> 10  # Stake
```

TypeScript targeting ES2020/CommonJS. Config at `~/.solclone/config.yml`, keypairs at `~/.solclone/id.json`.

### Flutter Wallet

```bash
cd flutter-wallet
flutter run -d chrome          # Web
flutter run -d linux           # Desktop
flutter run                    # Android (connected device)
```

Uses Provider for state management. Flutter 3.41+ required. Key deps: `bip39`, `ed25519_hd_key`, `pinenacl`, `qr_flutter`, `flutter_secure_storage`.

### Docker Multi-Network

```bash
docker compose --profile devnet up    # Validator + explorer + faucet
docker compose --profile testnet up
docker compose --profile full up      # All three networks simultaneously
```

### Wallet Packages

```bash
# wallet-standard — Wallet Standard cross-wallet compatibility
cd wallet-standard
npm install && npm run build     # Compile to dist/ (tsc)
npm test

# wallet-connect — WalletConnect v2 mobile QR pairing
cd wallet-connect
npm install && npm run build     # Compile to dist/ (tsc)
npm test

# connect-kit — React wallet connection components for DApps
cd connect-kit
npm install && npm run build     # Compile + bundle to dist/
npm run dev                      # Storybook for component development
npm test
```

### DeFi Suite (`defi/`)

Contains four sub-projects, each with a Rust on-chain program and some with a TypeScript UI:

```bash
# SolSwap DEX
cd defi/solswap/program && cargo build-sbf    # Build on-chain program
cd defi/solswap/ui && npm install && npm run dev  # AMM trading UI

# SolLend lending protocol
cd defi/sollend/program && cargo build-sbf
cd defi/sollend/ui && npm install && npm run dev

# SCUSD stablecoin
cd defi/scusd/program && cargo build-sbf

# Oracle price feeds
cd defi/oracle/program && cargo build-sbf
```

### NFT Marketplace (`nft-marketplace/`)

SolMart NFT marketplace with auctions and collections:

```bash
cd nft-marketplace/program && cargo build-sbf    # On-chain program
cd nft-marketplace/ui && npm install && npm run dev  # Marketplace UI
```

### Governance (`governance/`)

DAO governance with proposals, voting, and treasury:

```bash
cd governance/program && cargo build-sbf         # On-chain program
cd governance/ui && npm install && npm run dev    # Governance UI
```

### Faucet & Health Dashboard

```bash
# Faucet — devnet/testnet token faucet web UI
cd faucet && npm install && npm run dev          # Dev server

# Health Dashboard — real-time network stats
cd health-dashboard && npm install && npm run dev  # Dev server
```

## Testing

```bash
# Validator (Rust)
cd validator && cargo test --release --lib

# SPL programs
cd program-library && cargo test --release

# DeFi programs
cd defi/solswap/program && cargo test
cd defi/sollend/program && cargo test
cd defi/scusd/program && cargo test
cd defi/oracle/program && cargo test

# NFT marketplace program
cd nft-marketplace/program && cargo test

# Governance program
cd governance/program && cargo test

# Explorer
cd explorer && pnpm test              # vitest unit tests
cd explorer && pnpm test:e2e          # playwright E2E

# DApp scaffold
cd dapp-scaffold && npm test
```

The CLI wallet and Flutter wallet have no test suites currently.

## Architecture

All clients communicate with the validator via **Solana-compatible JSON-RPC** (default port 8899). The validator runs Proof of History + Tower BFT consensus and processes transactions through the Sealevel parallel runtime.

### Component Boundaries

- **`validator/`** — Rust workspace (100+ crates). Forked from `anza-xyz/agave`. Key crates: `cli/`, `core/` (consensus), `runtime/` (tx processing), `rpc/` (JSON-RPC server), `gossip/` (P2P), `ledger/` (storage). Build: `cargo build --release`.

- **`cli-wallet/`** — Standalone TypeScript CLI. Uses `commander`, `tweetnacl`, `bs58`. Builds its own raw JSON-RPC client in `src/lib/rpc-client.ts` — does **not** depend on `@solana/web3.js`. Commands in `src/commands/`, shared logic in `src/lib/`.

- **`flutter-wallet/`** — Cross-platform Dart app. State in `lib/providers/` (ChangeNotifier + Provider), network calls in `lib/services/rpc_service.dart`, screens in `lib/screens/`, reusable widgets in `lib/widgets/`. BIP39 mnemonics, Ed25519 HD derivation (m/44'/501'/0'/0').

- **`explorer/`** — Next.js app, pnpm, shadcn/ui. Forked from `solana-foundation/explorer`.

- **`program-library/`** — Rust workspace with SPL programs (token, governance, stake-pool, memo, etc.). Forked from `solana-labs/solana-program-library`.

- **`web3js-sdk/`** — TypeScript SDK using rollup. Forked from `solana-labs/solana-web3.js`.

- **`wallet-adapter/`** — React hooks/UI for DApp wallet integration (pnpm). `wallet-gui/` is Backpack wallet (yarn + turborepo).

- **`wallet-standard/`** — Wallet Standard implementation. Core files: `src/wallet.ts` (SolCloneWallet class), `src/adapter.ts` (legacy bridge), `src/register.ts` (registration), `src/detect.ts` (wallet detection). TypeScript, tsc build.

- **`wallet-connect/`** — WalletConnect v2 integration. Core files: `src/client.ts` (SignClient wrapper), `src/qr-modal.ts` (QR generation), `src/chains.ts` (SolClone chain IDs). TypeScript, tsc build.

- **`connect-kit/`** — React component library for DApp wallet connection. Core files: `src/provider.tsx` (SolCloneProvider), `src/components/ConnectButton.tsx`, `src/components/WalletModal.tsx`, `src/hooks/useWallet.ts`, `src/hooks/useSolClone.ts`. Composes wallet-standard + wallet-connect.

- **`defi/`** — DeFi suite with four sub-projects: `solswap/` (AMM DEX with program/ and ui/), `sollend/` (lending protocol with program/ and ui/), `scusd/` (algorithmic stablecoin, program/ only), `oracle/` (price feeds, program/ only). Rust programs build with `cargo build-sbf`, UIs are Next.js/React.

- **`nft-marketplace/`** — SolMart NFT marketplace. `program/` is the Rust on-chain program (listings, auctions, collections). `ui/` is the React marketplace frontend.

- **`governance/`** — DAO governance system. `program/` handles proposals, voting, and treasury management. `ui/` is the governance dashboard frontend.

- **`faucet/`** — Web UI for requesting devnet/testnet tokens. TypeScript, npm dev server.

- **`health-dashboard/`** — Real-time network stats dashboard showing TPS, slot height, validator count, epoch info. TypeScript, npm dev server.

### Networks

Genesis configs live in `networks/{devnet,testnet,mainnet}/genesis.json`.

| Network | RPC Port | WS Port | Faucet Port | Faucet Limit |
|---------|----------|---------|-------------|--------------|
| Devnet | 8899 | 8900 | 9900 | 5 SOL/req |
| Testnet | 8799 | 8800 | 9800 | 1 SOL/req |
| Mainnet | 8699 | 8700 | — | None |

### Production Ops (`ops/`)

- `ops/terraform/` — AWS provisioning (r6a.8xlarge, 2TB NVMe, security groups)
- `ops/ansible/` — Server configuration playbook
- `ops/validator/` — systemd service file, launch config, setup script
- `ops/monitoring/` — Prometheus + Grafana stack with validator dashboard and alerting rules
- `ops/network/` — `bootstrap-network.sh`, `add-validator.sh`, `genesis-ceremony.sh`
- `ops/security/` — UFW firewall script, hardening guide
- `ops/backup/` — S3 snapshot backup script
- `ops/runbooks/` — Incident response and upgrade procedures

## Split Repos

After modifying a forked component, sync to its split repo:

```bash
./scripts/update-split-repos.sh
```

Split repos: `code2031/solclone-validator`, `solclone-web3js`, `solclone-programs`, `solclone-explorer`, `solclone-wallet-adapter`, `solclone-backpack`, `solclone-dapp-scaffold`.

## Related Projects

- [ShardCoin](https://github.com/code2031/ShardCoin) — ShardCoin cryptocurrency
- [ShardWalletApp](https://github.com/code2031/ShardWalletApp) — ShardCoin Flutter wallet
