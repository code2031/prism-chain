# CLAUDE.md -- SolClone Privacy / Confidential Transfers

## Overview

Rust on-chain program implementing shielded pools for private token transfers on SolClone. Users deposit public SPL tokens into a pool, transfer privately within the pool using zero-knowledge proofs, and withdraw back to public accounts.

## Structure

```
privacy/
+-- program/
|   +-- src/
|   |   +-- lib.rs           # Program entry point and instruction dispatch
|   |   +-- state.rs         # Account structures (Pool, Commitment, Nullifier)
|   |   +-- instructions/    # Per-instruction logic
|   +-- Cargo.toml
+-- README.md                # Full architecture and privacy model docs
```

## Build & Test

```bash
cd privacy/program
cargo build-sbf           # Build the on-chain program
cargo test                # Run unit tests
```

## Program Instructions

| Instruction | Description |
|-------------|-------------|
| `initialize_pool` | Create a shielded pool for a specific SPL token mint |
| `shield` | Deposit public tokens into the pool, create a commitment |
| `transfer_shielded` | Transfer within the pool using ZK proofs (nullifiers in, commitments out) |
| `unshield` | Withdraw from the pool back to a public token account |

## Important Notes

- **ZK proof verification is currently a placeholder.** The program accepts any non-empty proof data. Production deployment requires integrating Groth16 (via `ark-groth16`) or PLONK.
- The Merkle tree uses a simplified hash. Production should use Poseidon hash for circuit compatibility.
- Encrypted note storage scheme is not yet specified; production needs ECIES.

## Key Files

- `program/src/lib.rs` -- Main program logic and instruction routing
- `program/src/state.rs` -- Pool state, commitment Merkle tree, nullifier set
- `README.md` -- Full architecture diagram, privacy model, and roadmap
