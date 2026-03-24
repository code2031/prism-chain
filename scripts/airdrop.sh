#!/usr/bin/env bash
# Quick airdrop script for local development
# Usage: ./scripts/airdrop.sh [amount] [address]

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="$ROOT/validator/target/release/solana"
AMOUNT="${1:-10}"
ADDRESS="${2:-}"

if [ ! -f "$CLI" ]; then
    echo "Error: CLI not built. Run 'make cli' first."
    exit 1
fi

if [ -z "$ADDRESS" ]; then
    ADDRESS=$("$CLI" address 2>/dev/null || echo "")
    if [ -z "$ADDRESS" ]; then
        echo "No address specified and no default keypair found."
        echo "Usage: $0 [amount] [address]"
        echo "   or: solana-keygen new"
        exit 1
    fi
fi

echo "Airdropping $AMOUNT SOL to $ADDRESS..."
"$CLI" airdrop "$AMOUNT" "$ADDRESS" --url http://localhost:8899

echo "New balance:"
"$CLI" balance "$ADDRESS" --url http://localhost:8899
