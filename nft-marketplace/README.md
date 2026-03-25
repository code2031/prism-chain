# SolMart -- NFT Marketplace

> Mint, list, buy, auction, and trade NFTs on the SolClone blockchain.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Overview

SolMart is a full-featured NFT marketplace built on SolClone. It provides an on-chain
Rust program for all marketplace operations and a Next.js web interface for browsing
collections, minting new NFTs, listing items for sale, placing bids in auctions, and
making offers. Creator royalties are enforced on-chain for every secondary sale.

## Features

- **Minting** -- Create NFTs with metadata, image URI, and optional collection assignment
- **Fixed-Price Listings** -- List NFTs at a set price in SCLONE tokens
- **English Auctions** -- Time-bound auctions with minimum bid increments
- **Offers** -- Make and accept offers on any NFT, even unlisted ones
- **Collections** -- Group NFTs into verified collections with on-chain association
- **Creator Royalties** -- Enforced royalty percentage (up to 10%) on every secondary sale
- **Activity Feed** -- Track mints, sales, listings, bids, and transfers in real time

## Marketplace Fee Structure

| Fee Type | Rate | Recipient |
|---|---|---|
| Marketplace Fee | 2% | Protocol treasury |
| Creator Royalty | 0-10% | Original creator |
| Listing Fee | Free | -- |

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

The SolMart interface will be available at `http://localhost:3000`.

## Program Instructions

| Instruction | Description |
|---|---|
| `mint_nft` | Mint a new NFT with metadata |
| `create_collection` | Initialize a verified collection |
| `list_nft` | List an NFT for fixed-price sale |
| `buy_nft` | Purchase a listed NFT |
| `create_auction` | Start an English auction for an NFT |
| `place_bid` | Place a bid on an active auction |
| `settle_auction` | Finalize auction and transfer NFT to winner |
| `make_offer` | Submit an offer on any NFT |
| `accept_offer` | Accept a pending offer |
| `cancel_listing` | Remove an NFT from sale |

## Tech Stack

- **On-Chain**: Rust, SolClone BPF program framework, Metaplex token metadata
- **Frontend**: Next.js 14, Tailwind CSS, wallet-adapter
- **Storage**: Metadata and images stored on Arweave / IPFS
- **Indexing**: Event-based indexer for marketplace activity

## License

Apache 2.0 -- see the root [LICENSE](../LICENSE) file.
