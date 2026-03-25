# CLAUDE.md -- SolClone Bridges

## Overview

Cross-chain bridge infrastructure with three bridge implementations (Ethereum, Bitcoin, Solana) and a unified UI. Each bridge has a Rust on-chain program and a TypeScript off-chain relayer or attestor. The privacy module at the repo root (`privacy/`) provides optional shielded pools for bridged assets.

## Structure

```
bridges/
+-- ethereum/
|   +-- program/     # Rust on-chain program (lock-and-mint for ERC-20)
|   +-- relayer/     # TypeScript relayer service
+-- bitcoin/
|   +-- program/     # Rust on-chain program (scBTC management)
|   +-- attestor/    # TypeScript multi-sig attestor service
+-- solana/
|   +-- program/     # Rust on-chain program (SPL token bridge)
|   +-- relayer/     # TypeScript relayer service
+-- ui/              # Next.js unified bridge interface
```

## Build Commands

### On-chain Programs

```bash
cd bridges/ethereum/program && cargo build-sbf
cd bridges/bitcoin/program && cargo build-sbf
cd bridges/solana/program && cargo build-sbf
```

### Off-chain Services

```bash
cd bridges/ethereum/relayer && npm install && npm start
cd bridges/bitcoin/attestor && npm install && npm start
cd bridges/solana/relayer && npm install && npm start
```

### Bridge UI

```bash
cd bridges/ui && npm install && npm run dev
```

## Testing

```bash
cd bridges/ethereum/program && cargo test
cd bridges/bitcoin/program && cargo test
cd bridges/solana/program && cargo test
```

## Key Files

- `ethereum/program/src/lib.rs` -- ERC-20 mint/burn logic
- `ethereum/relayer/src/index.ts` -- Ethereum event watcher + SolClone submitter
- `bitcoin/attestor/src/index.ts` -- Bitcoin TX monitor + multi-sig attestation
- `ui/app/page.tsx` -- Unified bridge interface entry point
- `privacy/program/src/lib.rs` -- Shielded pool program (repo root)
