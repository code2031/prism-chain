# SCUSD Stablecoin

SCUSD is an overcollateralized stablecoin pegged to $1 USD, backed by SCLONE token collateral on the SolClone network.

## How It Works

Users deposit SCLONE tokens into personal vaults and mint SCUSD against them. The system enforces a minimum **150% collateral ratio** to keep SCUSD fully backed.

### Key Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Minimum Collateral Ratio | 150% | Required ratio to mint or withdraw |
| Liquidation Ratio | 120% | Vaults below this can be liquidated |
| Stability Fee | 2% annual | Accrues on outstanding SCUSD debt |
| Liquidation Discount | 5% | Bonus collateral for liquidators |

### Instructions

- **initialize** -- Set up the SCUSD mint and global state.
- **open_vault** -- Create a personal collateral vault.
- **deposit_collateral** -- Add SCLONE collateral to your vault.
- **mint_scusd** -- Mint SCUSD against your collateral (must stay above 150%).
- **repay_scusd** -- Burn SCUSD to reduce your debt.
- **withdraw_collateral** -- Withdraw excess collateral (must stay above 150%).
- **liquidate_vault** -- Liquidate undercollateralized vaults (below 120%) and receive collateral at a 5% discount.

### Price Feed

SCUSD relies on the SolClone Price Oracle for SCLONE/USD prices. The oracle price is read as a `u64` with 8 decimal places of precision.

### Build

```bash
cargo build-bpf
```
