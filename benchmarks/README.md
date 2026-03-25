# SolClone Benchmark Suite

Performance benchmarks for the SolClone network measuring transactions per second (TPS) and transaction latency against a local test validator.

## What It Measures

| Benchmark | File | Metric |
|-----------|------|--------|
| **TPS** | `src/tps-bench.ts` | Maximum sustained transactions per second |
| **Latency** | `src/latency-bench.ts` | Transaction confirmation time (p50, p95, p99) |

## Quick Start

### Run All Benchmarks

```bash
npm install
bash run.sh
```

The `run.sh` script will:

1. Start a local SolClone test validator (if not already running)
2. Fund benchmark accounts via airdrop
3. Run the TPS benchmark
4. Run the latency benchmark
5. Generate a `RESULTS.md` report with timestamped results
6. Stop the test validator

### Run Individual Benchmarks

```bash
# TPS only
npx tsx src/tps-bench.ts

# Latency only
npx tsx src/latency-bench.ts
```

## Configuration

Edit `run.sh` or pass environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `RPC_URL` | `http://localhost:8899` | Validator RPC endpoint |
| `DURATION_SECS` | `30` | How long to sustain load (TPS bench) |
| `NUM_TRANSACTIONS` | `1000` | Number of transactions (latency bench) |
| `CONCURRENCY` | `50` | Parallel transaction senders |

## Output

After running, `RESULTS.md` is generated with a summary:

```
# SolClone Benchmark Results - 2026-03-24

## TPS Benchmark
- Sustained TPS: 4,250
- Peak TPS: 5,100
- Duration: 30s
- Total Transactions: 127,500

## Latency Benchmark
- p50: 420ms
- p95: 890ms
- p99: 1,250ms
- Transactions: 1,000
```

## Project Structure

```
benchmarks/
+-- src/
|   +-- tps-bench.ts        # TPS benchmark script
|   +-- latency-bench.ts    # Latency benchmark script
|   +-- report.ts           # RESULTS.md report generator
+-- run.sh                  # Orchestration script
+-- package.json            # Dependencies (tsx, @solana/web3.js)
```

## Prerequisites

- Node.js 18+
- SolClone validator (built from `validator/` or running via Docker)
- `tsx` is included as a dev dependency
