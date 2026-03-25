# SolClone DeFi Suite

Repository: https://github.com/code2031/solana-clone

## Build

```bash
# Build all on-chain programs
cargo build-bpf --workspace

# Build individual programs
cd solswap && cargo build-bpf
cd sollend && cargo build-bpf
cd scusd && cargo build-bpf
cd oracle && cargo build-bpf

# Run individual UIs
cd solswap/app && npm install && npm run dev   # :3000
cd sollend/app && npm install && npm run dev   # :3001
cd scusd/app && npm install && npm run dev     # :3002
cd oracle/app && npm install && npm run dev    # :3003

# Tests
cargo test --workspace                          # all Rust tests
cd solswap/app && npm test                      # SolSwap UI tests
```

## Key Files

- `solswap/src/lib.rs` -- SolSwap AMM program (x*y=k pools, 0.3% fee)
- `sollend/src/lib.rs` -- SolLend lending program (kinked rate curve, 85% LTV liquidation)
- `scusd/src/lib.rs` -- SCUSD stablecoin program (150% min collateral, $1 peg)
- `oracle/src/lib.rs` -- Price Oracle program (median aggregation, staleness checks)
- `solswap/app/` -- SolSwap Next.js frontend
- `sollend/app/` -- SolLend Next.js frontend
- `scusd/app/` -- SCUSD management frontend
- `oracle/app/` -- Oracle dashboard frontend
- `Cargo.toml` -- Workspace manifest for all four programs

## Architecture

Four independent Rust BPF programs sharing a Cargo workspace. Each program has its
own `src/lib.rs` entry point and a `app/` directory containing a Next.js frontend.

The Oracle program is the foundation: SolSwap uses it for reference pricing, SolLend
uses it for collateral valuation and liquidation triggers, and SCUSD uses it for
peg maintenance. All programs use u128 fixed-point math and validate account
ownership, signer authority, and PDA derivation on every instruction.
