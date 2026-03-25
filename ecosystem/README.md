# SolClone Ecosystem Tools

A collection of tools and platforms that support the SolClone ecosystem -- from token launches and validator discovery to on-chain identity and community funding.

## Components

| Directory | Description | Type |
|-----------|-------------|------|
| `launchpad/` | Fair token launch platform with multiple distribution mechanisms | Rust program + Next.js UI |
| `validator-marketplace/` | Browse, compare, and delegate to validators | Next.js UI |
| `profiles/` | On-chain identity and reputation system | Rust program |
| `grants/` | Ecosystem funding and grant management | Next.js UI |
| `bounties/` | Developer bounty board for ecosystem contributions | Next.js UI |

## Launchpad (`launchpad/`)

Fair token distribution platform supporting three launch types:

- **Fixed-Price** -- tokens sold at a set price until allocation is exhausted
- **Lottery** -- participants enter a lottery; winners get allocation at a fixed price
- **Auction** -- price discovery via ascending auction; all winners pay the clearing price

See `launchpad/README.md` for full program instructions and UI setup.

## Validator Marketplace (`validator-marketplace/`)

A web interface for stakers to discover and compare validators. Displays commission rates, uptime history, stake concentration, and APY estimates. Delegates can stake directly from the UI.

```bash
cd validator-marketplace && npm install && npm run dev
```

## Profiles (`profiles/`)

On-chain identity program that links wallets to human-readable names, avatars, and reputation scores. Other programs can query profiles for display names in UIs.

```bash
cd profiles/program && cargo build-sbf
```

## Grants (`grants/`)

Ecosystem funding platform where the SolClone Foundation and DAOs can post grant opportunities. Applicants submit proposals, reviewers score them, and funds are disbursed on-chain upon milestone completion.

```bash
cd grants && npm install && npm run dev
```

## Bounties (`bounties/`)

Developer bounty board for ecosystem contributions. Projects post bounties (bug fixes, features, integrations), developers claim and submit work, and reviewers approve payouts.

```bash
cd bounties && npm install && npm run dev
```

## Quick Start

```bash
# Build on-chain programs
cd launchpad/program && cargo build-sbf
cd profiles/program && cargo build-sbf

# Start any UI
cd launchpad/ui && npm install && npm run dev
cd validator-marketplace && npm install && npm run dev
cd grants && npm install && npm run dev
cd bounties && npm install && npm run dev
```

## Integration

Ecosystem tools interact with SolClone via standard JSON-RPC. The launchpad and profiles programs are deployed on devnet for testing. Validator marketplace reads live validator data from the cluster's `getVoteAccounts` RPC method.
