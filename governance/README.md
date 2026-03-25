# SolClone DAO Governance

> On-chain proposals, token-weighted voting, and treasury management for the SolClone ecosystem.

Part of the [SolClone](https://github.com/code2031/solana-clone) ecosystem.

---

## Overview

The SolClone Governance program enables decentralized decision-making for the protocol.
SCLONE token holders can create proposals, vote with token-weighted ballots, and execute
approved actions through a timelock. The system also manages the DAO treasury, allowing
the community to allocate funds for development, grants, and ecosystem growth.

## Features

- **Proposal Creation** -- Any token holder meeting the threshold can submit proposals
- **Token-Weighted Voting** -- Vote weight equals SCLONE token balance at snapshot slot
- **Quorum Enforcement** -- Proposals require minimum participation to pass
- **Timelock Execution** -- Approved proposals enter a delay period before execution
- **Treasury Management** -- DAO-controlled treasury for grants and protocol spending
- **Proposal Types** -- Parameter changes, treasury transfers, program upgrades, text proposals
- **Delegation** -- Delegate voting power to another address without transferring tokens
- **Vote Escrow** -- Tokens locked during active voting period

## Governance Parameters

| Parameter | Value |
|---|---|
| Proposal Threshold | 100,000 SCLONE |
| Quorum | 4% of total supply |
| Voting Period | 3 days (approx. 259,200 slots) |
| Timelock Delay | 2 days (approx. 172,800 slots) |
| Max Active Proposals | 10 |

## Proposal Lifecycle

1. **Draft** -- Author creates proposal with title, description, and executable actions
2. **Active** -- Voting opens; token holders cast For, Against, or Abstain votes
3. **Succeeded / Defeated** -- Voting closes; quorum and majority determine outcome
4. **Queued** -- Succeeded proposals enter the timelock queue
5. **Executed** -- After timelock delay, anyone can trigger execution
6. **Cancelled** -- Author or guardian can cancel before execution

## Quick Start

Build the on-chain program:

```bash
cargo build-bpf
```

Run the governance UI:

```bash
cd app
npm install
npm run dev
```

The governance dashboard will be available at `http://localhost:3000`.

## Program Instructions

| Instruction | Description |
|---|---|
| `create_governance` | Initialize a governance instance |
| `create_proposal` | Submit a new proposal |
| `cast_vote` | Cast a token-weighted vote on a proposal |
| `finalize_vote` | Close voting and determine outcome |
| `queue_proposal` | Move a succeeded proposal to the timelock |
| `execute_proposal` | Execute a queued proposal after timelock |
| `cancel_proposal` | Cancel a proposal before execution |
| `delegate_vote` | Delegate voting power to another address |

## Tech Stack

- **On-Chain**: Rust, SolClone BPF program framework
- **Frontend**: Next.js 14, Tailwind CSS, Recharts, wallet-adapter
- **Snapshots**: Token balances captured at proposal creation slot

## License

Apache 2.0 -- see the root [LICENSE](../LICENSE) file.
