# SolClone Price Oracle

Decentralized price feed oracle for the SolClone network. Aggregates prices from multiple authorized publishers using a median calculation.

## Supported Feeds

| Feed | Description |
|------|-------------|
| SCLONE/USD | Native SCLONE token price |
| SCUSD/USD | SCUSD stablecoin price (target $1.00) |
| BTC/USD | Bitcoin price |
| ETH/USD | Ethereum price |

## Price Format

All prices use **8 decimal places** of fixed-point precision. For example:
- `$1.00` = `100_000_000`
- `$45,123.99` = `4_512_399_000_000`

## Instructions

- **create_feed** -- Create a new price feed with a set of authorized publishers.
- **update_price** -- An authorized publisher submits a new price and confidence value.
- **aggregate_prices** -- Aggregate all recent publisher submissions using the median. Anyone can trigger aggregation.
- **add_publisher** / **remove_publisher** -- Feed authority manages the publisher list.

## Staleness

Submissions older than **120 slots** (~1 minute) are considered stale and excluded from aggregation.

## Build

```bash
cargo build-bpf
```
