# SolClone DAO Governance

Repository: https://github.com/code2031/solana-clone

## Build

```bash
# Build on-chain program
cargo build-bpf

# Run the governance UI
cd app && npm install && npm run dev   # starts on :3000

# Tests
cargo test                             # Rust program tests
cd app && npm test                     # frontend tests
npm run lint                           # ESLint
```

## Key Files

- `src/lib.rs` -- Main program entry point and instruction dispatcher
- `src/instructions/proposal.rs` -- Proposal creation, finalization, and cancellation
- `src/instructions/vote.rs` -- Vote casting with token-weight snapshot
- `src/instructions/timelock.rs` -- Timelock queue and execution logic
- `src/instructions/treasury.rs` -- Treasury transfer and management
- `src/instructions/delegation.rs` -- Vote delegation mechanics
- `src/state/` -- Account structures (governance, proposal, vote record, treasury)
- `app/page.tsx` -- Governance dashboard with active and past proposals
- `app/proposal/[id]/page.tsx` -- Proposal detail with voting interface
- `app/treasury/page.tsx` -- Treasury balance and spending history
- `app/delegate/page.tsx` -- Vote delegation management

## Architecture

Rust BPF on-chain program with a Next.js App Router frontend. Proposals store
executable instruction sets that run after timelock expiry. Vote weight is determined
by SCLONE token balance at the proposal creation slot, captured via a snapshot
mechanism. The timelock is enforced by comparing `Clock::unix_timestamp` against
the queued timestamp plus the configured delay.

The treasury is a PDA-controlled token account. Only executed proposals can
invoke treasury transfers. A guardian multisig can cancel proposals in emergencies
but cannot execute them directly.
