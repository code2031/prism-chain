# CLAUDE.md -- SolClone Benchmark Suite

## Overview

TPS and latency benchmarks for the SolClone validator. Benchmarks run against a local test validator and produce a timestamped RESULTS.md report.

## Quick Start

```bash
cd benchmarks
npm install
bash run.sh
```

The `run.sh` script starts a test validator, runs both benchmarks, generates RESULTS.md, and stops the validator.

## Individual Benchmarks

```bash
# TPS benchmark (sustained transaction throughput)
npx tsx src/tps-bench.ts

# Latency benchmark (confirmation time percentiles)
npx tsx src/latency-bench.ts
```

## Structure

```
benchmarks/
+-- src/
|   +-- tps-bench.ts        # TPS measurement
|   +-- latency-bench.ts    # Latency percentile measurement
|   +-- report.ts           # RESULTS.md generator
+-- run.sh                  # Full orchestration script
+-- package.json
```

## Configuration

Environment variables control benchmark parameters:

- `RPC_URL` -- Validator endpoint (default: `http://localhost:8899`)
- `DURATION_SECS` -- TPS benchmark duration (default: `30`)
- `NUM_TRANSACTIONS` -- Latency benchmark transaction count (default: `1000`)
- `CONCURRENCY` -- Parallel senders (default: `50`)

## Key Files

- `src/tps-bench.ts` -- Floods the validator with transfers, measures sustained TPS
- `src/latency-bench.ts` -- Sends transactions sequentially, records confirmation latency
- `src/report.ts` -- Aggregates results into Markdown report
- `run.sh` -- Orchestrates validator lifecycle and benchmark execution
