# SolClone Faucet

> Request devnet and testnet SCLONE tokens through a simple web interface.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Overview

The SolClone Faucet is a Next.js web application that allows developers to request
free SCLONE tokens on devnet and testnet networks. It provides a clean, branded UI
with built-in rate limiting to prevent abuse while keeping tokens accessible for
development and testing.

## Features

- **Network Selection** -- Toggle between devnet and testnet token requests
- **Wallet Address Input** -- Paste any valid SolClone address to receive tokens
- **Rate Limiting** -- 10 requests per hour per IP address, 5 requests per hour per wallet address
- **Transaction Tracking** -- View airdrop transaction signature and confirmation status
- **Responsive Design** -- Mobile-friendly UI with SolClone branding

## Quick Start

```bash
cd faucet
npm install
npm run dev
```

The faucet UI will be available at `http://localhost:3000`.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `FAUCET_RPC_URL` | `http://localhost:8899` | RPC endpoint for the target network |
| `FAUCET_AMOUNT` | `2` | Number of SCLONE tokens per request |
| `FAUCET_KEYPAIR` | `~/.config/solclone/faucet.json` | Funder keypair path |
| `RATE_LIMIT_IP` | `10` | Max requests per hour per IP |
| `RATE_LIMIT_ADDR` | `5` | Max requests per hour per address |

## API Endpoint

```
POST /api/airdrop
Content-Type: application/json

{
  "address": "<SCLONE_WALLET_ADDRESS>",
  "network": "devnet" | "testnet"
}
```

Returns a JSON response with the transaction signature and amount.

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Styling**: Tailwind CSS
- **Rate Limiting**: In-memory store with IP and address tracking
- **RPC Client**: SolClone web3.js SDK

## License

Apache 2.0 -- see the root [LICENSE](../LICENSE) file.
