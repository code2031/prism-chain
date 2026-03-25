# SolLend -- Lending and Borrowing Protocol

> Supply assets to earn interest or borrow against collateral on SolClone.

Part of the [SolClone DeFi Suite](../README.md) | [SolClone](https://github.com/code2031/solana-clone)

---

## Overview

SolLend is a decentralized lending protocol on SolClone. Users deposit supported
tokens to earn variable interest, or borrow tokens by posting collateral. The
interest rate follows a kinked curve model that incentivizes optimal utilization.
Positions that exceed 85% loan-to-value are eligible for liquidation.

## Features

- **Supply Markets** -- Deposit tokens to earn variable-rate interest
- **Borrowing** -- Borrow tokens against deposited collateral
- **Kinked Interest Rate Curve** -- Low rates at low utilization, steep increase above the kink
- **Liquidation Engine** -- Positions at or above 85% LTV can be liquidated by any caller
- **Liquidation Bonus** -- Liquidators receive a 5% bonus on repaid collateral
- **Oracle Integration** -- Collateral values sourced from SolClone Price Oracle
- **Health Factor Display** -- UI shows real-time position health

## Interest Rate Model

```
If utilization <= 80% (kink):
  rate = base_rate + (utilization / kink) * slope_1

If utilization > 80%:
  rate = base_rate + slope_1 + ((utilization - kink) / (1 - kink)) * slope_2
```

| Parameter | Value |
|---|---|
| Base Rate | 2% APY |
| Slope 1 | 10% APY |
| Slope 2 | 100% APY |
| Kink | 80% utilization |
| Liquidation LTV | 85% |
| Liquidation Bonus | 5% |

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

The SolLend interface will be available at `http://localhost:3001`.

## Program Instructions

| Instruction | Description |
|---|---|
| `init_reserve` | Create a lending reserve for a token |
| `deposit` | Supply tokens to a reserve and receive cTokens |
| `withdraw` | Redeem cTokens for underlying tokens |
| `borrow` | Borrow tokens against posted collateral |
| `repay` | Repay borrowed tokens to reduce debt |
| `liquidate` | Liquidate an unhealthy position (LTV >= 85%) |

## Tech Stack

- **On-Chain**: Rust, SolClone BPF program framework
- **Frontend**: Next.js 14, Tailwind CSS, Recharts, wallet-adapter
- **Price Data**: SolClone Price Oracle for collateral valuation

## License

Apache 2.0 -- see the root [LICENSE](../../LICENSE) file.
