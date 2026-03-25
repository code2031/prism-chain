# SolClone Bridges

Cross-chain bridge infrastructure enabling asset transfers between SolClone and external blockchains. Each bridge uses a lock-and-mint architecture where tokens are locked on the source chain and corresponding wrapped tokens are minted on SolClone.

## Supported Bridges

| Bridge | Directory | Mechanism | Estimated Transfer Time |
|--------|-----------|-----------|------------------------|
| **Ethereum** | `ethereum/` | ERC-20 lock-and-mint via relayer | ~15 minutes |
| **Bitcoin** | `bitcoin/` | Multi-sig scBTC with attestor network | ~60 minutes |
| **Solana** | `solana/` | SPL token bridge via relayer | ~30 seconds |

## Architecture

```
Source Chain                    SolClone
+------------------+           +-------------------+
| Lock tokens in   |  Relayer  | Mint wrapped      |
| bridge contract  | -------> | tokens on SolClone |
+------------------+  / Attest +-------------------+
                     /
             +------+------+
             | Attestor /  |
             | Relayer     |
             | (off-chain) |
             +-------------+
```

### Ethereum Bridge (`ethereum/`)

- **program/** -- Rust on-chain program that mints/burns wrapped ERC-20 tokens on SolClone
- **relayer/** -- TypeScript service that watches Ethereum events and submits mint instructions
- Lock ERC-20 tokens on Ethereum, receive scETH / scUSDC / scWETH on SolClone

### Bitcoin Bridge (`bitcoin/`)

- **program/** -- Rust on-chain program managing scBTC supply and redemptions
- **attestor/** -- TypeScript attestor service that monitors Bitcoin transactions via multi-sig
- Deposit BTC to a multi-sig address, receive scBTC on SolClone

### Solana Bridge (`solana/`)

- **program/** -- Rust on-chain program for SPL token bridging
- **relayer/** -- TypeScript relayer that watches Solana finality and relays proofs
- Bridge native SPL tokens between Solana mainnet and SolClone

## Unified Bridge UI (`ui/`)

A Next.js web application providing a single interface for all three bridges. Users select the source chain, token, and amount, and the UI handles the rest.

## Privacy Module

The `privacy/` module (at the repo root under `privacy/`) provides shielded pool functionality. Bridged tokens can optionally be deposited into shielded pools for private transfers within SolClone. See `privacy/README.md` for details.

## Quick Start

```bash
# Build bridge programs
cd ethereum/program && cargo build-sbf
cd bitcoin/program && cargo build-sbf
cd solana/program && cargo build-sbf

# Run the Ethereum relayer
cd ethereum/relayer && npm install && npm start

# Run the Bitcoin attestor
cd bitcoin/attestor && npm install && npm start

# Run the Solana relayer
cd solana/relayer && npm install && npm start

# Start the bridge UI
cd ui && npm install && npm run dev
```

## Security

All bridges require multi-party attestation before minting. Emergency pause functionality is built into each on-chain program. Bridge operators must be whitelisted via governance. See individual bridge READMEs for security details.
