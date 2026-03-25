# CLAUDE.md -- SolClone AI Features

## Overview

Four AI-powered Next.js applications that enhance the SolClone experience. Each is a standalone app in its own subdirectory.

## Applications

| App | Directory | Description |
|-----|-----------|-------------|
| **Portfolio Advisor** | `portfolio-advisor/` | AI risk analysis and portfolio rebalancing suggestions |
| **Contract Auditor** | `contract-auditor/` | Rust static analysis for SolClone programs (12 built-in rules) |
| **Explorer** | `explorer/` | Natural-language queries translated to JSON-RPC calls |
| **NFT Generator** | `nft-generator/` | Prompt-based NFT image generation and on-chain minting |

## Build & Run

```bash
# Each app runs independently
cd ai/portfolio-advisor && npm install && npm run dev
cd ai/contract-auditor && npm install && npm run dev
cd ai/explorer && npm install && npm run dev
cd ai/nft-generator && npm install && npm run dev
```

All apps start on port 3000 by default. Run them on different ports with `PORT=3001 npm run dev`.

## Contract Auditor Rules

The auditor checks Rust source files against 12 static analysis rules covering:
- Unchecked arithmetic / integer overflow
- Missing signer checks
- Missing owner checks
- Uninitialized account reads
- PDA seed collisions
- Unsafe deserialization
- Missing rent exemption checks
- Duplicate mutable accounts
- Authority escalation
- Unbounded allocations
- Missing close account logic
- Cross-program invocation reentrancy

## Architecture Notes

- All four apps are Next.js with the App Router
- AI inference calls go through API routes in `app/api/`
- Portfolio advisor connects to the SolClone RPC for live token balances
- Explorer converts NL queries to RPC method calls (`getTransaction`, `getBalance`, etc.)
- NFT generator calls an image generation API, then mints via Metaplex

## Testing

Each app has its own test setup. Run `npm test` inside any app directory.
