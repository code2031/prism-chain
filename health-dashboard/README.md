# SolClone Health Dashboard

> Real-time network monitoring and statistics for the SolClone blockchain.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Overview

The SolClone Health Dashboard is a Next.js application that provides live visibility
into network health. It displays current slot height, transactions per second, active
validator count, epoch progress, and historical performance charts. Data is polled
from the RPC endpoint every 2 seconds using SWR for efficient cache-based revalidation.

## Features

- **Slot Height** -- Live current slot with blocks-per-second rate
- **TPS Gauge** -- Real-time transactions per second with peak tracking
- **Validator Table** -- Active, delinquent, and total validator counts with stake distribution
- **Epoch Progress** -- Current epoch number, slot index, and estimated time remaining
- **Performance Charts** -- Historical TPS, slot time, and skip rate graphs via Recharts
- **Network Selector** -- Switch between mainnet, devnet, and testnet views
- **Auto-Refresh** -- Polls RPC every 2 seconds with SWR stale-while-revalidate

## Quick Start

```bash
cd health-dashboard
npm install
npm run dev
```

The dashboard will be available at `http://localhost:3000`.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `RPC_URL` | `http://localhost:8899` | SolClone RPC endpoint |
| `POLL_INTERVAL` | `2000` | Polling interval in milliseconds |
| `HISTORY_WINDOW` | `300` | Number of data points to retain for charts |

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Charts**: Recharts for time-series and gauge visualizations
- **Data Fetching**: SWR with 2-second refresh interval
- **Styling**: Tailwind CSS with SolClone design tokens
- **RPC Client**: SolClone web3.js SDK

## RPC Methods Used

- `getSlot` -- Current slot height
- `getRecentPerformanceSamples` -- TPS and slot timing
- `getVoteAccounts` -- Validator status and stake
- `getEpochInfo` -- Epoch number, slot index, slots in epoch

## License

Apache 2.0 -- see the root [LICENSE](../LICENSE) file.
