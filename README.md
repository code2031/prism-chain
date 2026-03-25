# SolClone

A high-performance Layer 1 blockchain forked from [Solana](https://github.com/anza-xyz/agave). Complete ecosystem including validator, CLI wallet, Flutter GUI wallet, block explorer, SDK, SPL programs, DApp framework, and multi-network infrastructure.

> **[Read the Whitepaper](./WHITEPAPER.md)**

---

## Architecture

```
                          ┌─────────────────────────────────────────┐
                          │              SolClone Network            │
                          │                                         │
   ┌──────────┐          │  ┌───────────┐    ┌─────────────────┐  │
   │ CLI      │──RPC────▶│  │ Validator  │◀──▶│  Gossip Network │  │
   │ Wallet   │          │  │  (PoH +    │    │  (P2P Protocol) │  │
   └──────────┘          │  │  Tower BFT)│    └─────────────────┘  │
                          │  └─────┬─────┘                         │
   ┌──────────┐          │        │                                │
   │ Flutter  │──RPC────▶│  ┌─────▼─────┐    ┌─────────────────┐  │
   │ Wallet   │          │  │  Runtime   │───▶│  SPL Programs   │  │
   └──────────┘          │  │ (Sealevel) │    │  (Token, Stake) │  │
                          │  └─────┬─────┘    └─────────────────┘  │
   ┌──────────┐          │        │                                │
   │ Explorer │──RPC────▶│  ┌─────▼─────┐                         │
   │ (Next.js)│          │  │ Accounts   │                         │
   └──────────┘          │  │(Cloudbreak)│                         │
                          │  └───────────┘                         │
   ┌──────────┐          │                                         │
   │  DApps   │──RPC────▶│       ┌──────────┐                     │
   │(web3.js) │          │       │  Faucet   │ (devnet/testnet)   │
   └──────────┘          │       └──────────┘                     │
                          └─────────────────────────────────────────┘
```

## Components

| Directory | Description | Language |
|-----------|-------------|----------|
| `validator/` | Blockchain core — validator, CLI, runtime, PoH, gossip, RPC | Rust |
| `cli-wallet/` | Custom CLI wallet with full token + staking support | TypeScript |
| `flutter-wallet/` | Cross-platform GUI wallet (Android, iOS, Web, Desktop) | Dart/Flutter |
| `explorer/` | Block explorer web app | Next.js |
| `web3js-sdk/` | JavaScript/TypeScript SDK for DApp developers | TypeScript |
| `program-library/` | SPL programs — Token, Token-2022, Governance, Stake Pool, Memo | Rust |
| `wallet-adapter/` | React hooks/UI for connecting wallets to DApps | TypeScript |
| `wallet-standard/` | Wallet Standard for cross-wallet compatibility | TypeScript |
| `wallet-connect/` | WalletConnect v2 mobile QR pairing | TypeScript |
| `connect-kit/` | React wallet connection components for DApps | TypeScript/React |
| `wallet-gui/` | Backpack wallet (alternative GUI wallet) | React Native |
| `dapp-scaffold/` | DApp starter template | Next.js |
| `defi/` | DeFi suite: SolSwap DEX, SolLend, SCUSD stablecoin, Oracle | Rust/TypeScript |
| `nft-marketplace/` | SolMart NFT marketplace with auctions and collections | Rust/TypeScript |
| `governance/` | DAO governance with proposals, voting, treasury | Rust/TypeScript |
| `faucet/` | Devnet/testnet faucet web UI | TypeScript |
| `health-dashboard/` | Real-time network stats dashboard | TypeScript |
| `networks/` | Genesis configs for devnet, testnet, mainnet | JSON |
| `metaplex/` | Metaplex Token Metadata — NFT standard | Rust |
| `anchor/` | Anchor framework — easiest way to build programs | Rust |
| `docker/` | Docker images for validator, explorer, faucet | Docker |
| `ops/` | Production ops — Terraform, Ansible, Prometheus, Grafana, systemd | IaC |
| `docs/` | Tokenomics, API reference, validator guide, developer guide | Markdown |
| `branding/` | Logo, brand guidelines, color palette | SVG/Markdown |
| `ai/` | AI features: portfolio advisor, contract auditor, NL explorer, NFT generator | Next.js |
| `bridges/` | Cross-chain bridges: Ethereum, Bitcoin, Solana + unified UI | Rust/TypeScript |
| `ecosystem/` | Ecosystem tools: launchpad, validator marketplace, profiles, grants, bounties | Rust/TypeScript |
| `playground/` | Browser-based Solana program IDE and simulator | Next.js |
| `templates/` | Anchor program templates: token, NFT, escrow, voting, staking | Rust |
| `benchmarks/` | TPS and latency benchmark suite | TypeScript |
| `docs-site/` | Docusaurus documentation site | TypeScript |
| `privacy/` | Confidential transfers with shielded pools and ZK proofs | Rust |
| `examples/` | Example scripts: create token, create NFT, deploy program, run DApp | Bash |

## Third-Party Wallet Support

SolClone implements the [Wallet Standard](https://github.com/wallet-standard/wallet-standard), which means **any compliant wallet works out of the box** with SolClone DApps -- no custom integration required.

| Wallet | Connection Method |
|--------|-------------------|
| **Phantom** | Browser extension (Wallet Standard) |
| **Solflare** | Browser extension (Wallet Standard) |
| **Backpack** | Browser extension (Wallet Standard) |
| **Any Wallet Standard wallet** | Auto-detected via `@solclone/wallet-standard` |
| **Any mobile wallet** | QR code pairing via `@solclone/wallet-connect` |

DApp developers can add full wallet support in three lines of code using `@solclone/connect-kit`:

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

See [`wallet-standard/`](./wallet-standard/), [`wallet-connect/`](./wallet-connect/), and [`connect-kit/`](./connect-kit/) for details.

## What You Can Do

```bash
# Create a fungible token
./examples/create-token.sh MyToken 9

# Create an NFT (supply=1, immutable)
./examples/create-nft.sh MyNFT

# Deploy a custom program
./examples/deploy-program.sh <program.so>

# Quick-start: network + explorer + DApp in one command
./examples/run-dapp.sh
```

## Networks

SolClone runs three networks, matching Solana's architecture:

| Network | RPC Port | Faucet | Use Case |
|---------|----------|--------|----------|
| **Devnet** | `8899` | 5 SOL/request | DApp development, testing |
| **Testnet** | `8799` | 1 SOL/request | Validator testing, staging |
| **Mainnet** | `8699` | None | Production |
| **Localnet** | `8899` | Unlimited | Local single-node development |

## Quick Start

### Option 1: One-command setup

```bash
./scripts/setup.sh --all
```

### Option 2: Step by step

```bash
# Check dependencies
make check-deps

# Build the validator + CLI (takes 10-30 min)
make validator

# Start local testnet
make testnet-bg

# Airdrop yourself some SOL
./scripts/airdrop.sh 100

# Start the explorer
make explorer
```

### Option 3: Docker (fastest)

```bash
# Start devnet (validator + explorer + faucet)
docker compose --profile devnet up

# Or start all networks
docker compose --profile full up
```

### Option 4: CLI Wallet

```bash
cd cli-wallet && npm install && npm run build

# Generate a keypair
npx solclone keygen

# Check balance
npx solclone balance

# Request airdrop (devnet)
npx solclone airdrop 5

# Send tokens
npx solclone transfer <address> 1.5
```

### Option 5: Flutter Wallet

```bash
cd flutter-wallet

# Run on web
flutter run -d chrome

# Run on Android
flutter run

# Run on desktop
flutter run -d linux
```

## Make Commands

```
make help            # Show all commands
make check-deps      # Verify prerequisites
make setup           # Install all JS dependencies
make validator       # Build validator (release)
make cli             # Build CLI tools only
make testnet         # Start local testnet (foreground)
make testnet-bg      # Start local testnet (background)
make stop-testnet    # Stop background testnet
make explorer        # Start block explorer
make wallet          # Start Backpack wallet
make dapp            # Start DApp scaffold
make sdk             # Build web3.js SDK
make programs        # Build all SPL programs
make status          # Show build/run status
make clean           # Clean all artifacts
make all             # Build everything
```

## Project Structure

```
solclone/
├── validator/            # Solana validator (Rust) — forked from anza-xyz/agave
│   ├── cli/              #   Solana CLI tool
│   ├── core/             #   Consensus, PoH, Tower BFT
│   ├── runtime/          #   Transaction processing (Sealevel)
│   ├── rpc/              #   JSON-RPC server
│   ├── gossip/           #   P2P gossip protocol
│   └── ledger/           #   Ledger storage
├── cli-wallet/           # Custom TypeScript CLI wallet
│   └── src/commands/     #   keygen, balance, transfer, stake, token...
├── flutter-wallet/       # Cross-platform Flutter GUI wallet
│   └── lib/              #   screens, services, providers, widgets
├── explorer/             # Block explorer (Next.js)
├── web3js-sdk/           # JavaScript/TypeScript SDK
├── program-library/      # SPL programs (Rust)
│   ├── token/            #   SPL Token program
│   ├── governance/       #   On-chain governance
│   └── stake-pool/       #   Stake pool program
├── wallet-adapter/       # React wallet adapter
├── wallet-standard/      # Wallet Standard cross-wallet compatibility
├── wallet-connect/       # WalletConnect v2 mobile QR pairing
├── connect-kit/          # React wallet connection components
├── wallet-gui/           # Backpack wallet (React Native)
├── dapp-scaffold/        # DApp starter (Next.js)
├── defi/                 # DeFi suite (SolSwap, SolLend, SCUSD, Oracle)
├── nft-marketplace/      # SolMart NFT marketplace
├── governance/           # DAO governance (proposals, voting, treasury)
├── faucet/               # Devnet/testnet faucet web UI
├── health-dashboard/     # Real-time network stats dashboard
├── networks/             # Network configurations
│   ├── devnet/           #   Devnet genesis config
│   ├── testnet/          #   Testnet genesis config
│   └── mainnet/          #   Mainnet genesis config
├── docker/               # Docker images
├── scripts/              # Setup, airdrop, deploy scripts
├── .github/workflows/    # CI/CD pipeline
├── docker-compose.yml    # Multi-network Docker setup
├── Makefile              # Unified build system
└── WHITEPAPER.md         # Technical whitepaper
```

## Prerequisites

| Tool | Required For | Install |
|------|-------------|---------|
| Rust 1.75+ | Validator, SPL programs | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js 18+ | Explorer, SDK, DApps, CLI wallet | [nodejs.org](https://nodejs.org) |
| pnpm | Explorer, wallet-adapter | `npm i -g pnpm` |
| yarn | Backpack wallet | `npm i -g yarn` |
| Flutter 3.x | Flutter wallet | [flutter.dev](https://flutter.dev) |
| Docker | Containerized networks | [docker.com](https://docker.com) |

## Production Deployment

```bash
# Provision AWS validator (Terraform)
cd ops/terraform && terraform init && terraform apply

# Configure the server (Ansible)
cd ops/ansible && ansible-playbook -i inventory playbook.yml

# Bootstrap a new network
./ops/network/bootstrap-network.sh --cluster devnet

# Add a validator to an existing network
./ops/network/add-validator.sh --cluster devnet --entrypoint <ip>:8001

# Start monitoring (Prometheus + Grafana)
cd ops/monitoring && docker compose -f docker-compose.monitoring.yml up -d
# Grafana: http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
```

See `docs/validator-guide.md` for the full operator guide and `networks/mainnet/validator-requirements.md` for hardware specs.

## Documentation

| Document | Description |
|----------|-------------|
| [Whitepaper](./WHITEPAPER.md) | Technical architecture, consensus, tokenomics |
| [Tokenomics](./docs/tokenomics.md) | SCLONE supply, distribution, inflation, staking |
| [API Reference](./docs/api-reference.md) | Complete JSON-RPC API documentation |
| [Validator Guide](./docs/validator-guide.md) | Operator setup, monitoring, troubleshooting |
| [Developer Guide](./docs/developer-guide.md) | Building programs, deploying DApps, SDK usage |
| [Brand Guidelines](./branding/brand-guidelines.md) | Colors, typography, logo, voice |
| [Security Policy](./SECURITY.md) | Vulnerability reporting, responsible disclosure |

## Upstream Sources

| Component | Forked From |
|-----------|-------------|
| Validator | [anza-xyz/agave](https://github.com/anza-xyz/agave) |
| Web3.js SDK | [solana-labs/solana-web3.js](https://github.com/solana-labs/solana-web3.js) |
| SPL Programs | [solana-labs/solana-program-library](https://github.com/solana-labs/solana-program-library) |
| Explorer | [solana-foundation/explorer](https://github.com/solana-foundation/explorer) |
| Wallet Adapter | [anza-xyz/wallet-adapter](https://github.com/anza-xyz/wallet-adapter) |
| Backpack Wallet | [coral-xyz/backpack](https://github.com/coral-xyz/backpack) |
| DApp Scaffold | [solana-labs/dapp-scaffold](https://github.com/solana-labs/dapp-scaffold) |

## License

Each component retains its original license. See individual directories for details. Custom components (cli-wallet, flutter-wallet, networks, scripts) are MIT licensed.
