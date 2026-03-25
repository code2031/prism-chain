# SolClone Launchpad

A fair token distribution platform built as a Rust on-chain program with a Next.js frontend. Supports three launch types to accommodate different project needs.

## Architecture

```
launchpad/
+-- program/       # Rust on-chain program (Anchor-compatible)
|   +-- src/
|   |   +-- lib.rs          # Program entry point and instruction dispatch
|   |   +-- state.rs        # Account structures (Launch, Participant)
|   |   +-- instructions/   # Per-instruction logic
|   +-- Cargo.toml
+-- ui/            # Next.js frontend
    +-- app/
    +-- components/
    +-- lib/
    +-- package.json
```

## Launch Types

| Type | Mechanism | Best For |
|------|-----------|----------|
| **Fixed-Price** | Tokens sold at a set price, first-come-first-served until allocation runs out | Simple, predictable launches |
| **Lottery** | Participants deposit entry fee; winners selected randomly, losers refunded | Fair access for high-demand launches |
| **Auction** | Ascending price auction; all winners pay the final clearing price | Price discovery for new tokens |

## Program Instructions

| Instruction | Description |
|-------------|-------------|
| `create_launch` | Initialize a new token launch with type, allocation, price, and schedule |
| `participate` | Deposit funds to participate in a launch |
| `finalize` | End the launch, determine winners (lottery), set clearing price (auction) |
| `claim` | Winners claim their token allocation after finalization |
| `refund` | Non-winners (lottery) or cancelled launches get deposits returned |
| `withdraw` | Launch creator withdraws collected funds after finalization |

## Quick Start

### Build the Program

```bash
cd program
cargo build-sbf
```

### Deploy to Devnet

```bash
solana program deploy target/deploy/launchpad.so \
  --url devnet \
  --keypair ~/.solclone/id.json
```

### Run the UI

```bash
cd ui
npm install
npm run dev
# Open http://localhost:3000
```

## Creating a Launch (CLI Example)

```bash
# Fixed-price launch: 1,000,000 tokens at 0.01 SOL each
solclone launchpad create \
  --type fixed-price \
  --token <MINT_ADDRESS> \
  --allocation 1000000 \
  --price 0.01 \
  --start "2026-04-01T00:00:00Z" \
  --end "2026-04-07T00:00:00Z"
```

## Testing

```bash
cd program
cargo test
```

Tests cover all three launch types, edge cases (oversubscription, early finalization), and refund logic.
