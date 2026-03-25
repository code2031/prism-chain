# SolMart NFT Marketplace

Repository: https://github.com/code2031/solana-clone

## Build

```bash
# Build on-chain program
cargo build-bpf

# Run the web UI
cd app && npm install && npm run dev   # starts on :3000

# Tests
cargo test                             # Rust program tests
cd app && npm test                     # frontend tests
npm run lint                           # ESLint
```

## Key Files

- `src/lib.rs` -- Main program entry point and instruction dispatcher
- `src/instructions/mint.rs` -- NFT minting with metadata assignment
- `src/instructions/listing.rs` -- Fixed-price listing and purchase logic
- `src/instructions/auction.rs` -- English auction creation, bidding, and settlement
- `src/instructions/offer.rs` -- Offer creation and acceptance
- `src/instructions/collection.rs` -- Collection creation and NFT association
- `src/state/` -- Account data structures (listing, auction, offer, collection)
- `app/page.tsx` -- Marketplace home with featured collections and recent activity
- `app/mint/page.tsx` -- NFT minting form
- `app/nft/[id]/page.tsx` -- Individual NFT detail, buy, bid, and offer pages
- `app/collection/[id]/page.tsx` -- Collection browser with floor price and volume

## Architecture

Rust BPF on-chain program with a Next.js App Router frontend. The program stores
listing, auction, and offer state in PDAs derived from the NFT mint address. Royalty
enforcement happens at the program level during every transfer that involves payment.

The frontend uses wallet-adapter for transaction signing and an event-based indexer
to populate the activity feed and collection stats. Metadata and images are referenced
via Arweave/IPFS URIs stored in the Metaplex token metadata account.
