# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Prism is a complete Solana blockchain fork. This monorepo contains the validator (Rust), CLI wallet (TypeScript), Flutter GUI wallet (Dart), block explorer (Next.js), web3.js SDK, SPL programs, wallet adapter, DApp scaffold, and production operations tooling. The 7 forked upstream components each have their own split repo under `code2031/prism-*`.

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
npx prism keygen            # Generate Ed25519 keypair
npx prism config use devnet # Switch network
npx prism balance           # Check SOL balance
npx prism transfer <addr> 1 # Send SOL
npx prism token create-mint # Create SPL token
npx prism stake delegate <validator> 10  # Stake
```

TypeScript targeting ES2020/CommonJS. Config at `~/.prism/config.yml`, keypairs at `~/.prism/id.json`.

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

### AI Features (`ai/`)

Four Next.js apps for AI-powered blockchain tools:

```bash
cd ai/portfolio-advisor && npm install && npm run dev  # Risk analysis
cd ai/contract-auditor && npm install && npm run dev   # Rust static analysis (12 rules)
cd ai/explorer && npm install && npm run dev           # NL queries -> RPC
cd ai/nft-generator && npm install && npm run dev      # Prompt-based NFT minting
```

### Cross-Chain Bridges (`bridges/`)

Three bridge programs (Rust) with off-chain relayers/attestors (TypeScript):

```bash
# Build bridge programs
cd bridges/ethereum/program && cargo build-sbf
cd bridges/bitcoin/program && cargo build-sbf
cd bridges/solana/program && cargo build-sbf

# Run relayers/attestors
cd bridges/ethereum/relayer && npm install && npm start
cd bridges/bitcoin/attestor && npm install && npm start
cd bridges/solana/relayer && npm install && npm start

# Bridge UI
cd bridges/ui && npm install && npm run dev
```

### Ecosystem Tools (`ecosystem/`)

```bash
# Build on-chain programs
cd ecosystem/launchpad/program && cargo build-sbf
cd ecosystem/profiles/program && cargo build-sbf

# Run UIs
cd ecosystem/launchpad/ui && npm install && npm run dev
cd ecosystem/validator-marketplace && npm install && npm run dev
cd ecosystem/grants && npm install && npm run dev
cd ecosystem/bounties && npm install && npm run dev
```

### Playground (`playground/`)

Browser-based program IDE:

```bash
cd playground && npm install && npm run dev
```

### Anchor Templates (`templates/`)

Five Anchor program templates (token, nft-collection, escrow, voting, staking-pool):

```bash
cd templates/<name>/programs && cargo build-sbf
```

### Benchmarks (`benchmarks/`)

```bash
cd benchmarks && npm install
bash run.sh                        # Full suite: starts validator, runs benchmarks, generates RESULTS.md
npx tsx src/tps-bench.ts           # TPS only
npx tsx src/latency-bench.ts       # Latency only
```

### Docs Site (`docs-site/`)

Docusaurus documentation site:

```bash
cd docs-site && npm install
npm start                          # Dev server
npm run build                      # Production build
```

### Privacy / Confidential Transfers (`privacy/`)

Shielded pool program for private token transfers:

```bash
cd privacy/program && cargo build-sbf
cd privacy/program && cargo test
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

```bash
# Bridge programs
cd bridges/ethereum/program && cargo test
cd bridges/bitcoin/program && cargo test
cd bridges/solana/program && cargo test

# Ecosystem programs
cd ecosystem/launchpad/program && cargo test
cd ecosystem/profiles/program && cargo test

# Privacy program
cd privacy/program && cargo test

# Benchmarks (run full suite)
cd benchmarks && bash run.sh
```

The CLI wallet and Flutter wallet have no test suites currently.

## Architecture

All clients communicate with the validator via **Solana-compatible JSON-RPC** (default port 8899). The validator runs Proof of History + Tower BFT consensus and processes transactions through the Sealevel parallel runtime.

### Component Boundaries

- **`validator/`** — Rust workspace (100+ crates). Forked from `anza-xyz/agave`. Key crates: `cli/`, `core/` (consensus), `runtime/` (tx processing), `rpc/` (JSON-RPC server), `gossip/` (P2P), `ledger/` (storage). Build: `cargo build --release`.

- **`cli-wallet/`** — Standalone TypeScript CLI. Uses `commander`, `tweetnacl`, `bs58`. Builds its own raw JSON-RPC client in `src/lib/rpc-client.ts` — does **not** depend on `@solana/web3.js`. Commands in `src/commands/`, shared logic in `src/lib/`.

- **`flutter-wallet/`** — Cross-platform Dart app. State in `lib/providers/` (ChangeNotifier + Provider), network calls in `lib/services/rpc_service.dart`, screens in `lib/screens/`, reusable widgets in `lib/widgets/`. BIP39 mnemonics, Ed25519 HD derivation (m/44'/501'/0'/0'). Supports 17 chains via `lib/services/chains/`.

- **`explorer/`** — Next.js app, pnpm, shadcn/ui. Forked from `solana-foundation/explorer`.

- **`program-library/`** — Rust workspace with SPL programs (token, governance, stake-pool, memo, etc.). Forked from `solana-labs/solana-program-library`.

- **`web3js-sdk/`** — TypeScript SDK using rollup. Forked from `solana-labs/solana-web3.js`.

- **`wallet-adapter/`** — React hooks/UI for DApp wallet integration (pnpm). `wallet-gui/` is Backpack wallet (yarn + turborepo).

- **`wallet-standard/`** — Wallet Standard implementation. Core files: `src/wallet.ts` (PrismWallet class), `src/adapter.ts` (legacy bridge), `src/register.ts` (registration), `src/detect.ts` (wallet detection). TypeScript, tsc build.

- **`wallet-connect/`** — WalletConnect v2 integration. Core files: `src/client.ts` (SignClient wrapper), `src/qr-modal.ts` (QR generation), `src/chains.ts` (Prism chain IDs). TypeScript, tsc build.

- **`connect-kit/`** — React component library for DApp wallet connection. Core files: `src/provider.tsx` (PrismProvider), `src/components/ConnectButton.tsx`, `src/components/WalletModal.tsx`, `src/hooks/useWallet.ts`, `src/hooks/usePrism.ts`. Composes wallet-standard + wallet-connect.

- **`defi/`** — DeFi suite with four sub-projects: `solswap/` (AMM DEX with program/ and ui/), `sollend/` (lending protocol with program/ and ui/), `scusd/` (algorithmic stablecoin, program/ only), `oracle/` (price feeds, program/ only). Rust programs build with `cargo build-sbf`, UIs are Next.js/React.

- **`nft-marketplace/`** — SolMart NFT marketplace. `program/` is the Rust on-chain program (listings, auctions, collections). `ui/` is the React marketplace frontend.

- **`governance/`** — DAO governance system. `program/` handles proposals, voting, and treasury management. `ui/` is the governance dashboard frontend.

- **`faucet/`** — Web UI for requesting devnet/testnet tokens. TypeScript, npm dev server.

- **`health-dashboard/`** — Real-time network stats dashboard showing TPS, slot height, validator count, epoch info. TypeScript, npm dev server.

- **`ai/`** — Four AI-powered Next.js apps: `portfolio-advisor/` (risk analysis), `contract-auditor/` (Rust static analysis with 12 built-in rules), `explorer/` (NL queries translated to RPC calls), `nft-generator/` (prompt-based NFT minting). Each is a standalone Next.js App Router project.

- **`bridges/`** — Cross-chain bridge infrastructure. Three bridges: `ethereum/` (ERC-20 lock-and-mint, ~15 min), `bitcoin/` (multi-sig scBTC, ~60 min), `solana/` (SPL token bridge, ~30 sec). Each has `program/` (Rust) and `relayer/` or `attestor/` (TypeScript). `ui/` is the unified Next.js bridge interface.

- **`ecosystem/`** — Ecosystem tools. `launchpad/` (Rust program + Next.js UI for token launches: fixed-price, lottery, auction), `validator-marketplace/` (validator browsing UI), `profiles/` (on-chain identity program), `grants/` (ecosystem funding UI), `bounties/` (developer bounty board UI).

- **`playground/`** — Browser-based Solana program IDE and simulator. Next.js app.

- **`templates/`** — Five Anchor program templates: `token/`, `nft-collection/`, `escrow/`, `voting/`, `staking-pool/`. Each has `programs/src/lib.rs` + `Cargo.toml` + `README.md`. Used via `prism init <name> --template <type>`.

- **`benchmarks/`** — TPS and latency benchmark suite. `src/tps-bench.ts` and `src/latency-bench.ts` run against a local test validator. `run.sh` orchestrates the full suite and generates `RESULTS.md`.

- **`docs-site/`** — Docusaurus documentation site. 7 guides: intro, getting-started, cli-reference, sdk-guide, anchor-guide, defi-guide, nft-guide. Config in `docusaurus.config.ts`, sidebar in `sidebars.ts`.

- **`privacy/`** — Confidential transfer program with shielded pools. Instructions: `initialize_pool`, `shield`, `transfer_shielded`, `unshield`. ZK proof verification is placeholder (production: Groth16/PLONK). Merkle tree of commitments + nullifier set.

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

Split repos: `code2031/prism-validator`, `prism-web3js`, `prism-programs`, `prism-explorer`, `prism-wallet-adapter`, `prism-backpack`, `prism-dapp-scaffold`.

## Related Projects

- [ShardCoin](https://github.com/code2031/ShardCoin) — ShardCoin cryptocurrency
- [ShardWalletApp](https://github.com/code2031/ShardWalletApp) — ShardCoin Flutter wallet
