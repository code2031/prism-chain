# SolSwap -- Automated Market Maker DEX

> Constant-product AMM for trustless token swaps on SolClone.

Part of the [SolClone DeFi Suite](../README.md) | [SolClone](https://github.com/code2031/solana-clone)

---

## Overview

SolSwap is a decentralized exchange built on the constant-product invariant (x * y = k).
Users can create liquidity pools for any SPL token pair, provide liquidity to earn fees,
and swap tokens permissionlessly. A 0.3% fee is charged on every swap and distributed
pro-rata to liquidity providers.

## Features

- **Pool Creation** -- Permissionless pool initialization for any token pair
- **Token Swaps** -- Instant swaps using the constant-product x*y=k formula
- **Liquidity Provision** -- Deposit token pairs to receive LP tokens representing pool share
- **Fee Distribution** -- 0.3% swap fee accrues to LP token holders automatically
- **Slippage Protection** -- User-configurable maximum slippage tolerance
- **Price Impact Display** -- UI shows estimated price impact before confirming

## Swap Fee Breakdown

| Recipient | Share | Description |
|---|---|---|
| Liquidity Providers | 0.25% | Accrues in pool reserves |
| Protocol Treasury | 0.05% | Funds SolClone DAO governance |

## Quick Start

Build the on-chain program:

```bash
cargo build-bpf
```

Run the web UI:

```bash
cd app
npm install
npm run dev
```

The SolSwap interface will be available at `http://localhost:3000`.

## Program Instructions

| Instruction | Description |
|---|---|
| `initialize_pool` | Create a new liquidity pool for a token pair |
| `add_liquidity` | Deposit tokens and receive LP tokens |
| `remove_liquidity` | Burn LP tokens and withdraw underlying tokens |
| `swap` | Exchange one token for another through a pool |

## Tech Stack

- **On-Chain**: Rust, SolClone BPF program framework
- **Frontend**: Next.js 14, Tailwind CSS, wallet-adapter
- **Math**: u128 fixed-point arithmetic to prevent overflow

## Security Considerations

- All pool operations verify signer authority and account ownership
- Swap amounts are validated against minimum output to prevent sandwich attacks
- LP token mint authority is a PDA owned by the program

## License

Apache 2.0 -- see the root [LICENSE](../../LICENSE) file.
