#!/usr/bin/env bash
# ============================================================================
# Solana Clone — One-command setup
# Usage: ./scripts/setup.sh [--all | --validator | --js | --docker]
# ============================================================================

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
RESET='\033[0m'

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

info()  { echo -e "${CYAN}▸ $1${RESET}"; }
ok()    { echo -e "${GREEN}✓ $1${RESET}"; }
warn()  { echo -e "${YELLOW}! $1${RESET}"; }
fail()  { echo -e "${RED}✗ $1${RESET}"; }

# ── Dependency checks ──────────────────────────────────────────────────────

check_deps() {
    info "Checking prerequisites..."

    local missing=0

    if command -v rustc &>/dev/null; then
        ok "Rust $(rustc --version | awk '{print $2}')"
    else
        fail "Rust not found"
        echo "    Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        missing=1
    fi

    if command -v node &>/dev/null; then
        ok "Node.js $(node --version)"
    else
        fail "Node.js not found"
        echo "    Install: https://nodejs.org"
        missing=1
    fi

    if command -v pnpm &>/dev/null; then
        ok "pnpm $(pnpm --version)"
    else
        warn "pnpm not found (needed for explorer) — installing..."
        npm install -g pnpm
        ok "pnpm installed"
    fi

    if command -v yarn &>/dev/null; then
        ok "yarn $(yarn --version 2>/dev/null || echo 'installed')"
    else
        warn "yarn not found (needed for wallet) — installing..."
        npm install -g yarn
        ok "yarn installed"
    fi

    return $missing
}

# ── Install JS dependencies ────────────────────────────────────────────────

setup_js() {
    info "Installing JavaScript dependencies..."

    cd "$ROOT/web3js-sdk"
    npm install --legacy-peer-deps 2>/dev/null && ok "web3js-sdk" || warn "web3js-sdk failed"

    cd "$ROOT/explorer"
    pnpm install 2>/dev/null && ok "explorer" || warn "explorer failed"

    cd "$ROOT/dapp-scaffold"
    npm install --legacy-peer-deps 2>/dev/null && ok "dapp-scaffold" || warn "dapp-scaffold failed"

    cd "$ROOT/wallet-adapter"
    pnpm install 2>/dev/null && ok "wallet-adapter" || warn "wallet-adapter failed"
}

# ── Build validator ────────────────────────────────────────────────────────

setup_validator() {
    info "Building Solana validator (this will take 10-30 minutes)..."
    cd "$ROOT/validator"
    cargo build --release 2>&1 | tail -5
    ok "Validator built"
}

# ── Configure CLI ──────────────────────────────────────────────────────────

configure_cli() {
    local CLI="$ROOT/validator/target/release/solana"
    if [ -f "$CLI" ]; then
        info "Configuring CLI for local testnet..."
        "$CLI" config set --url http://localhost:8899 2>/dev/null || true
        ok "CLI configured (RPC: http://localhost:8899)"
    fi
}

# ── Main ───────────────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  ╔═══════════════════════════════════════╗"
    echo "  ║       Solana Clone Setup Script        ║"
    echo "  ╚═══════════════════════════════════════╝"
    echo ""

    local mode="${1:---all}"

    check_deps || {
        fail "Missing required dependencies. Install them and re-run."
        exit 1
    }
    echo ""

    case "$mode" in
        --all)
            setup_js
            echo ""
            setup_validator
            configure_cli
            ;;
        --js)
            setup_js
            ;;
        --validator)
            setup_validator
            configure_cli
            ;;
        *)
            echo "Usage: $0 [--all | --validator | --js]"
            exit 1
            ;;
    esac

    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════${RESET}"
    echo -e "${GREEN}  Setup complete! Next steps:${RESET}"
    echo ""
    echo "  Start local testnet:    make testnet"
    echo "  Start explorer:         make explorer"
    echo "  Start DApp scaffold:    make dapp"
    echo "  Check status:           make status"
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════${RESET}"
}

main "$@"
