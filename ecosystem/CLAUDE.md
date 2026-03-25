# CLAUDE.md -- SolClone Ecosystem Tools

## Overview

Five sub-projects supporting the SolClone ecosystem. Two have on-chain Rust programs (launchpad, profiles). Three are UI-only (validator-marketplace, grants, bounties).

## Structure

```
ecosystem/
+-- launchpad/
|   +-- program/     # Rust on-chain program (token launches)
|   +-- ui/          # Next.js launch UI
+-- validator-marketplace/   # Next.js validator browsing and delegation
+-- profiles/
|   +-- program/     # Rust on-chain identity program
+-- grants/          # Next.js grant management UI
+-- bounties/        # Next.js bounty board UI
```

## Build Commands

### On-chain Programs

```bash
cd ecosystem/launchpad/program && cargo build-sbf
cd ecosystem/profiles/program && cargo build-sbf
```

### UIs

```bash
cd ecosystem/launchpad/ui && npm install && npm run dev
cd ecosystem/validator-marketplace && npm install && npm run dev
cd ecosystem/grants && npm install && npm run dev
cd ecosystem/bounties && npm install && npm run dev
```

## Testing

```bash
cd ecosystem/launchpad/program && cargo test
cd ecosystem/profiles/program && cargo test
```

## Key Files

- `launchpad/program/src/lib.rs` -- Launch creation, participation, finalization, claims, refunds
- `profiles/program/src/lib.rs` -- On-chain identity: names, avatars, reputation
- `validator-marketplace/app/page.tsx` -- Validator listing with commission, uptime, APY
- `grants/app/page.tsx` -- Grant proposals and milestone tracking
- `bounties/app/page.tsx` -- Bounty postings, claims, and payouts
