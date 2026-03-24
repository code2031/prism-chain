#!/usr/bin/env bash
# Deploy a compiled Solana program to the local testnet
# Usage: ./scripts/deploy-program.sh <path-to-program.so>

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="$ROOT/validator/target/release/solana"
PROGRAM_PATH="${1:-}"

if [ ! -f "$CLI" ]; then
    echo "Error: CLI not built. Run 'make cli' first."
    exit 1
fi

if [ -z "$PROGRAM_PATH" ]; then
    echo "Usage: $0 <path-to-program.so>"
    echo ""
    echo "Example:"
    echo "  $0 program-library/target/deploy/spl_token.so"
    exit 1
fi

if [ ! -f "$PROGRAM_PATH" ]; then
    echo "Error: Program file not found: $PROGRAM_PATH"
    exit 1
fi

echo "Deploying program: $PROGRAM_PATH"
"$CLI" program deploy "$PROGRAM_PATH" --url http://localhost:8899

echo "Done."
