# SolClone DeFi Suite

> A complete decentralized finance ecosystem built on the SolClone blockchain.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Overview

The SolClone DeFi Suite is a collection of composable on-chain programs and web
interfaces that together provide a full-featured DeFi stack. Each component is
independently deployable but designed to interoperate through shared token standards
and the SolClone Price Oracle.

## Components

| Module | Directory | Description |
|---|---|---|
| **SolSwap** | `solswap/` | Automated market maker DEX with constant-product pools |
| **SolLend** | `sollend/` | Lending and borrowing protocol with kinked interest rates |
| **SCUSD** | `scusd/` | Over-collateralized stablecoin pegged to $1 USD |
| **Oracle** | `oracle/` | Decentralized price feeds with median aggregation |

## Architecture

```
Oracle (price feeds)
  |
  +---> SolSwap (DEX pools, uses oracle for reference prices)
  |
  +---> SolLend (lending, uses oracle for collateral valuation)
  |       |
  |       +---> Liquidation engine (triggers at 85% LTV)
  |
  +---> SCUSD (stablecoin, uses oracle for peg maintenance)
```

All programs are written in Rust using the SolClone program framework and compile
to BPF bytecode for on-chain execution. Each module includes a Next.js web UI for
end-user interaction.

## Quick Start

Build all programs from this directory:

```bash
cd defi
cargo build-bpf --workspace
```

Or build and run individual component UIs:

```bash
cd solswap && npm install && npm run dev   # DEX UI on :3000
cd sollend && npm install && npm run dev   # Lending UI on :3001
cd scusd && npm install && npm run dev     # Stablecoin UI on :3002
cd oracle && npm install && npm run dev    # Oracle dashboard on :3003
```

## Shared Dependencies

- **SolClone web3.js SDK** -- RPC and transaction building
- **SPL Token** -- Token minting, transfers, and account management
- **SolClone Price Oracle** -- On-chain price feeds consumed by all protocols

## Security

All programs have been designed with reentrancy protection, signer verification, and
account ownership checks. See [SECURITY.md](../SECURITY.md) for the disclosure policy.

## License

Apache 2.0 -- see the root [LICENSE](../LICENSE) file.
