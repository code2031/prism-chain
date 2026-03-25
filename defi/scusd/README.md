# SCUSD -- SolClone Stablecoin

> Over-collateralized stablecoin pegged to $1 USD, backed by SCLONE deposits.

Part of the [SolClone DeFi Suite](../README.md) | [SolClone](https://github.com/code2031/solana-clone)

---

## Overview

SCUSD is a decentralized, over-collateralized stablecoin on the SolClone blockchain.
Users mint SCUSD by depositing SCLONE tokens as collateral at a minimum 150%
collateralization ratio. The peg to $1 USD is maintained through a combination of
collateral requirements, liquidation mechanics, and arbitrage incentives.

## Features

- **Mint SCUSD** -- Deposit SCLONE tokens to mint SCUSD at 150% minimum collateralization
- **Burn and Redeem** -- Burn SCUSD to unlock deposited SCLONE collateral
- **Dynamic Collateral Ratio** -- Oracle-fed SCLONE price determines real-time ratio
- **Liquidation** -- Vaults below 150% collateralization can be liquidated
- **Stability Fee** -- Annual fee accrued on minted SCUSD, paid on redemption
- **Global Debt Ceiling** -- Protocol-level cap on total SCUSD supply

## Peg Mechanism

| Scenario | Market Response |
|---|---|
| SCUSD > $1 | Users mint new SCUSD and sell, pushing price down |
| SCUSD < $1 | Users buy cheap SCUSD and burn to redeem collateral at $1 value |

## Key Parameters

| Parameter | Value |
|---|---|
| Minimum Collateral Ratio | 150% |
| Liquidation Threshold | 150% |
| Liquidation Penalty | 10% |
| Stability Fee | 2% APY |
| Debt Ceiling | 10,000,000 SCUSD |

## Quick Start

Build the on-chain program:

```bash
cargo build-bpf
```

Deploy to devnet:

```bash
solclone program deploy target/deploy/scusd.so --url devnet
```

## Program Instructions

| Instruction | Description |
|---|---|
| `create_vault` | Open a new collateral vault |
| `deposit_collateral` | Add SCLONE collateral to a vault |
| `mint_scusd` | Mint SCUSD against vault collateral |
| `burn_scusd` | Burn SCUSD to reduce vault debt |
| `withdraw_collateral` | Remove excess collateral from a vault |
| `liquidate_vault` | Liquidate an under-collateralized vault |

## Tech Stack

- **On-Chain**: Rust, SolClone BPF program framework
- **Price Data**: SolClone Price Oracle for SCLONE/USD price
- **Math**: u128 fixed-point with 6-decimal precision

## License

Apache 2.0 -- see the root [LICENSE](../../LICENSE) file.
