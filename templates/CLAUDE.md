# CLAUDE.md -- SolClone Anchor Templates

## Overview

Five pre-built Anchor program templates at `templates/{token,nft-collection,escrow,voting,staking-pool}/`. Each is a complete, buildable project with a Rust program and a README.

## Structure

Each template follows the same layout:

```
templates/<name>/
+-- programs/
|   +-- src/
|   |   +-- lib.rs       # Program entry point, instructions, accounts
|   +-- Cargo.toml       # Dependencies (anchor-lang, anchor-spl)
+-- README.md
```

## Build Commands

```bash
# Build any template
cd templates/<name>/programs && cargo build-sbf

# Examples
cd templates/token/programs && cargo build-sbf
cd templates/nft-collection/programs && cargo build-sbf
cd templates/escrow/programs && cargo build-sbf
cd templates/voting/programs && cargo build-sbf
cd templates/staking-pool/programs && cargo build-sbf
```

## Template Summary

| Template | Key Instructions |
|----------|-----------------|
| `token` | `initialize_mint`, `mint_to`, `transfer`, `burn` |
| `nft-collection` | `create_collection`, `mint_nft`, `update_metadata` |
| `escrow` | `initialize_escrow`, `accept`, `cancel` |
| `voting` | `create_proposal`, `cast_vote`, `finalize` |
| `staking-pool` | `initialize_pool`, `stake`, `unstake`, `claim_rewards` |

## CLI Integration

```bash
solclone init my-project --template token
```

This copies the template, renames the project in Cargo.toml, and generates a new program keypair.

## Key Files

- `<name>/programs/src/lib.rs` -- All program logic for each template
- `<name>/programs/Cargo.toml` -- Rust dependencies
- `<name>/README.md` -- Template-specific usage docs
