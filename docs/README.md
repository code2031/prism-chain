# Prism Documentation

Core technical documentation for the Prism blockchain. For the rendered documentation site, see `docs-site/`.

## Contents

| File | Description |
|------|-------------|
| `api-reference.md` | Full JSON-RPC API reference with method signatures, parameters, and curl examples. Covers all standard RPC methods (getBalance, getBlock, getTransaction, sendTransaction, etc.). |
| `developer-guide.md` | End-to-end guide for building on Prism: environment setup, Rust toolchain, program development, deployment, testing, and SDK usage. |
| `tokenomics.md` | PRISM token economics: 500M initial supply, distribution (40% validators, 30% ecosystem, 20% foundation, 10% community), vesting schedules, and inflation model. |
| `validator-guide.md` | Validator operator guide: hardware requirements (256 GB RAM, 2 TB NVMe), setup, keypair management, staking, monitoring, and maintenance. |

## Internal Documents

The `superpowers/` directory contains internal planning materials:

```
superpowers/
+-- plans/
|   +-- 2026-03-24-phase1-foundation.md
+-- specs/
    +-- 2026-03-24-solclone-masterplan-design.md
```

## Key Reference

- **RPC Endpoint**: `http://localhost:8899` (default)
- **Amounts**: All values in lamports (1 PRISM = 1,000,000,000 lamports)
- **Public Keys**: Base58-encoded strings
- **Commitment Levels**: `finalized`, `confirmed`, `processed`
