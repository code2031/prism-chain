# SolClone Anchor Templates

Five pre-built Anchor program templates for common on-chain use cases. Each template is a complete, buildable Anchor project with program logic, account structures, and a README.

## Available Templates

| Template | Directory | Description |
|----------|-----------|-------------|
| **Token** | `token/` | Fungible token with mint, transfer, and burn |
| **NFT Collection** | `nft-collection/` | NFT collection with metadata and royalties |
| **Escrow** | `escrow/` | Two-party token escrow with timeout |
| **Voting** | `voting/` | On-chain proposal creation and weighted voting |
| **Staking Pool** | `staking-pool/` | Token staking with time-based rewards |

## Usage

### Initialize a New Project from a Template

```bash
solclone init my-project --template token
```

This copies the template into `my-project/`, updates `Cargo.toml` with your project name, and generates a fresh program ID.

### Manual Usage

You can also copy a template directory directly:

```bash
cp -r templates/escrow my-escrow
cd my-escrow/programs
cargo build-sbf
```

## Template Structure

Each template follows the same layout:

```
<template>/
+-- programs/
|   +-- src/
|   |   +-- lib.rs       # Program entry point, instructions, accounts
|   +-- Cargo.toml       # Rust dependencies (anchor-lang, anchor-spl)
+-- README.md            # Template-specific documentation
```

## Template Details

### Token (`token/`)

Minimal fungible token program. Instructions: `initialize_mint`, `mint_to`, `transfer`, `burn`. Demonstrates SPL Token CPI and PDA authority patterns.

### NFT Collection (`nft-collection/`)

NFT minting with on-chain metadata. Instructions: `create_collection`, `mint_nft`, `update_metadata`. Integrates with Metaplex Token Metadata for standard-compliant NFTs.

### Escrow (`escrow/`)

Two-party atomic swap escrow. Instructions: `initialize_escrow`, `accept`, `cancel`. Includes timeout-based cancellation so funds are never permanently locked.

### Voting (`voting/`)

Governance-style voting. Instructions: `create_proposal`, `cast_vote`, `finalize`. Supports token-weighted voting and configurable quorum thresholds.

### Staking Pool (`staking-pool/`)

Token staking with linear reward distribution. Instructions: `initialize_pool`, `stake`, `unstake`, `claim_rewards`. Reward rate is configurable by the pool authority.

## Building a Template

```bash
cd templates/<name>/programs
cargo build-sbf
```

## Prerequisites

- Rust 1.75+
- Anchor CLI (`cargo install --git https://github.com/coral-xyz/anchor anchor-cli`)
- Solana CLI tools
