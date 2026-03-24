# SolClone Networks

SolClone runs three separate networks, just like Solana:

| Network | Purpose | RPC URL | Explorer |
|---------|---------|---------|----------|
| **Mainnet** | Production network with real value | `https://api.mainnet.solclone.io` | `https://explorer.solclone.io` |
| **Testnet** | Staging environment for validators | `https://api.testnet.solclone.io` | `https://explorer.solclone.io?cluster=testnet` |
| **Devnet** | Development and testing (free airdrop) | `https://api.devnet.solclone.io` | `https://explorer.solclone.io?cluster=devnet` |
| **Localnet** | Local single-node for development | `http://localhost:8899` | `http://localhost:3000` |

## Running Locally

```bash
# Start a local devnet
make network-devnet

# Start a local testnet
make network-testnet

# Start just a single-node localnet
make testnet
```

## Network Differences

### Devnet
- Free SOL via airdrop faucet (max 5 SOL per request)
- Reset periodically
- Ideal for DApp development and testing
- Relaxed rate limits

### Testnet
- Mirrors mainnet configuration
- Validator testing and staking experiments
- Limited faucet (1 SOL per request)
- Longer history retention

### Mainnet
- Production network
- Real economic value
- No faucet
- Full validator set
