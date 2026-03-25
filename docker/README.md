# Prism Docker

Dockerfiles and supporting files for containerized Prism components. Used by the root `docker-compose.yml` to run multi-network environments.

## Files

| File | Description |
|------|-------------|
| `Dockerfile.validator` | Multi-stage Rust build of the validator. Produces `solana-validator`, `solana-test-validator`, and `solana` CLI binaries. Exposes ports 8899, 8900, 8001, 8002. |
| `Dockerfile.explorer` | Multi-stage Node.js/pnpm build of the Next.js block explorer. Exposes port 3000. |
| `Dockerfile.faucet` | Lightweight Node.js image running `faucet-server.js`. Exposes port 9900. |
| `faucet-server.js` | Express-based faucet API server with rate limiting. Proxies `requestAirdrop` RPC calls to the validator. |

## Usage

These Dockerfiles are referenced by the root `docker-compose.yml`:

```bash
# From the repo root:
docker compose --profile devnet up      # Validator + explorer + faucet
docker compose --profile testnet up
docker compose --profile full up        # All three networks
```

To build individually:

```bash
docker build -f docker/Dockerfile.validator -t prism-validator .
docker build -f docker/Dockerfile.explorer -t prism-explorer .
docker build -f docker/Dockerfile.faucet -t prism-faucet .
```

## Faucet Configuration

The faucet server reads these environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `CLUSTER` | `devnet` | Target network (`devnet`, `testnet`, `mainnet`) |
| `RPC_URL` | `http://validator:8899` | Validator RPC endpoint |
| `PORT` | `9900` | Faucet listen port |

Rate limits: devnet allows 5 SOL/req at 10 req/hr, testnet allows 1 SOL/req at 3 req/hr. Mainnet airdrops are disabled.
