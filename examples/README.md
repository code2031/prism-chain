# Prism Examples

Runnable shell scripts demonstrating common Prism operations. Each script is self-contained -- it checks for prerequisites, requests airdrops when needed, and prints step-by-step output.

## Scripts

| Script | Description |
|--------|-------------|
| `run-dapp.sh` | Full quick-start: launches a local test validator, funds a wallet, and prints instructions for running the explorer, DApp scaffold, and wallets. |
| `create-token.sh` | Creates an SPL token: mints a new token, creates an associated account, and mints 1,000,000 tokens. |
| `create-nft.sh` | Creates an NFT: mints a token with 0 decimals and supply of 1, then disables further minting. |
| `deploy-program.sh` | Deploys a compiled `.so` program to the local testnet. Prints build instructions for Anchor and cargo-build-sbf. |

## Usage

All scripts assume the validator CLI has been built (`make cli`).

```bash
# Start local network and get set up
./examples/run-dapp.sh

# Create an SPL token with 9 decimals
./examples/create-token.sh MyToken 9

# Create a non-fungible token
./examples/create-nft.sh MyNFT

# Deploy a compiled program
./examples/deploy-program.sh target/deploy/my_program.so
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RPC_URL` | `http://localhost:8899` | Target RPC endpoint |

## Prerequisites

- Validator CLI built: `make cli` (produces `validator/target/release/solana` and `spl-token`)
- A running test validator: `make testnet-bg`
