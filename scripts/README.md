# Prism Scripts

Repository-level utility scripts for development setup, local operations, and split repo management.

## Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `setup.sh` | One-command project setup. Checks prerequisites (Rust, Node.js, pnpm, yarn), installs JS dependencies for web3js-sdk/explorer/dapp-scaffold/wallet-adapter, optionally builds the validator, and configures the CLI. | `./scripts/setup.sh [--all \| --js \| --validator]` |
| `airdrop.sh` | Requests an airdrop on the local testnet. | `./scripts/airdrop.sh [amount] [address]` |
| `deploy-program.sh` | Deploys a compiled `.so` program to the local testnet. | `./scripts/deploy-program.sh <path-to-program.so>` |
| `split-repos.sh` | Initial push of the 7 forked components to their individual GitHub repos under `code2031/prism-*`. Force-pushes fresh git histories. | `./scripts/split-repos.sh` |
| `update-split-repos.sh` | Incremental update of the 7 split repos. Only pushes if there are changes. | `./scripts/update-split-repos.sh` |

## Split Repo Mapping

The split/update scripts manage these component-to-repo mappings:

| Directory | GitHub Repo |
|-----------|-------------|
| `validator` | `code2031/prism-validator` |
| `web3js-sdk` | `code2031/prism-web3js` |
| `program-library` | `code2031/prism-programs` |
| `explorer` | `code2031/prism-explorer` |
| `wallet-adapter` | `code2031/prism-wallet-adapter` |
| `wallet-gui` | `code2031/prism-backpack` |
| `dapp-scaffold` | `code2031/prism-dapp-scaffold` |

## Prerequisites

- Validator CLI built (`make cli`) for airdrop and deploy scripts
- A running local test validator (`make testnet-bg`) for airdrop and deploy
- GitHub authentication configured for split repo scripts
