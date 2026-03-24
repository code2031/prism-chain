# ============================================================================
# Solana Clone - Unified Build System
# ============================================================================

.PHONY: help setup check-deps validator cli testnet explorer wallet dapp sdk \
        programs clean stop-testnet status all

SHELL := /bin/bash
ROOT := $(shell pwd)
VALIDATOR_BIN := $(ROOT)/validator/target/release

# Colors
GREEN  := \033[0;32m
YELLOW := \033[0;33m
CYAN   := \033[0;36m
RED    := \033[0;31m
RESET  := \033[0m

help: ## Show this help
	@echo ""
	@echo "  Solana Clone - Build System"
	@echo "  ==========================="
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  $(CYAN)%-20s$(RESET) %s\n", $$1, $$2}'
	@echo ""

# ── Prerequisites ──────────────────────────────────────────────────────────

check-deps: ## Check all required dependencies
	@echo "$(CYAN)Checking dependencies...$(RESET)"
	@command -v rustc  >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) Rust $$(rustc --version | awk '{print $$2}')" || echo "  $(RED)✗$(RESET) Rust not found — install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
	@command -v cargo  >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) Cargo" || echo "  $(RED)✗$(RESET) Cargo not found"
	@command -v node   >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) Node.js $$(node --version)" || echo "  $(RED)✗$(RESET) Node.js not found — install: https://nodejs.org"
	@command -v npm    >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) npm $$(npm --version)" || echo "  $(RED)✗$(RESET) npm not found"
	@command -v pnpm   >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) pnpm $$(pnpm --version)" || echo "  $(YELLOW)○$(RESET) pnpm not found — install: npm i -g pnpm (needed for explorer)"
	@command -v yarn   >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) yarn $$(yarn --version 2>/dev/null)" || echo "  $(YELLOW)○$(RESET) yarn not found — install: npm i -g yarn (needed for wallet-gui)"
	@command -v docker >/dev/null 2>&1 && echo "  $(GREEN)✓$(RESET) Docker $$(docker --version | awk '{print $$3}' | tr -d ',')" || echo "  $(YELLOW)○$(RESET) Docker not found (optional)"
	@echo ""

setup: check-deps ## Install all JS dependencies
	@echo "$(CYAN)Installing dependencies for all JS projects...$(RESET)"
	@cd $(ROOT)/web3js-sdk && npm install 2>/dev/null && echo "  $(GREEN)✓$(RESET) web3js-sdk"
	@cd $(ROOT)/explorer && pnpm install 2>/dev/null && echo "  $(GREEN)✓$(RESET) explorer" || echo "  $(YELLOW)○$(RESET) explorer (install pnpm first)"
	@cd $(ROOT)/dapp-scaffold && npm install 2>/dev/null && echo "  $(GREEN)✓$(RESET) dapp-scaffold"
	@cd $(ROOT)/wallet-adapter && pnpm install 2>/dev/null && echo "  $(GREEN)✓$(RESET) wallet-adapter" || echo "  $(YELLOW)○$(RESET) wallet-adapter (install pnpm first)"
	@echo "$(GREEN)Setup complete.$(RESET)"

# ── Validator / Blockchain Core ────────────────────────────────────────────

validator: ## Build the Solana validator (release mode)
	@echo "$(CYAN)Building validator (this takes a while)...$(RESET)"
	cd $(ROOT)/validator && cargo build --release
	@echo "$(GREEN)✓ Validator built at $(VALIDATOR_BIN)/solana-validator$(RESET)"

cli: ## Build the Solana CLI tools
	@echo "$(CYAN)Building CLI tools...$(RESET)"
	cd $(ROOT)/validator && cargo build --release -p solana-cli
	@echo "$(GREEN)✓ CLI built at $(VALIDATOR_BIN)/solana$(RESET)"

testnet: ## Start a local test validator
	@echo "$(CYAN)Starting local test validator...$(RESET)"
	@echo "  RPC:       http://localhost:8899"
	@echo "  Websocket: ws://localhost:8900"
	@echo "  Logs:      $(ROOT)/.testnet.log"
	@echo ""
	@if [ -f $(VALIDATOR_BIN)/solana-test-validator ]; then \
		$(VALIDATOR_BIN)/solana-test-validator \
			--log $(ROOT)/.testnet.log \
			--rpc-port 8899; \
	else \
		echo "$(RED)Validator not built yet. Run 'make validator' first.$(RESET)"; \
	fi

testnet-bg: ## Start local test validator in background
	@if [ -f $(ROOT)/.testnet.pid ]; then \
		echo "$(YELLOW)Testnet already running (PID $$(cat $(ROOT)/.testnet.pid))$(RESET)"; \
	elif [ -f $(VALIDATOR_BIN)/solana-test-validator ]; then \
		$(VALIDATOR_BIN)/solana-test-validator \
			--log $(ROOT)/.testnet.log \
			--rpc-port 8899 &> /dev/null & \
		echo $$! > $(ROOT)/.testnet.pid; \
		echo "$(GREEN)✓ Testnet started (PID $$!)$(RESET)"; \
		echo "  RPC: http://localhost:8899"; \
	else \
		echo "$(RED)Validator not built yet. Run 'make validator' first.$(RESET)"; \
	fi

stop-testnet: ## Stop the background test validator
	@if [ -f $(ROOT)/.testnet.pid ]; then \
		kill $$(cat $(ROOT)/.testnet.pid) 2>/dev/null; \
		rm -f $(ROOT)/.testnet.pid; \
		echo "$(GREEN)✓ Testnet stopped$(RESET)"; \
	else \
		echo "$(YELLOW)No testnet running$(RESET)"; \
	fi

# ── SPL Programs ───────────────────────────────────────────────────────────

programs: ## Build all SPL programs (Token, Governance, etc.)
	@echo "$(CYAN)Building SPL programs...$(RESET)"
	cd $(ROOT)/program-library && cargo build --release
	@echo "$(GREEN)✓ SPL programs built$(RESET)"

token-program: ## Build just the SPL Token program
	@echo "$(CYAN)Building SPL Token program...$(RESET)"
	cd $(ROOT)/program-library && cargo build --release -p spl-token
	@echo "$(GREEN)✓ Token program built$(RESET)"

# ── Web / JS Projects ─────────────────────────────────────────────────────

explorer: ## Start the block explorer (localhost:3000)
	@echo "$(CYAN)Starting block explorer...$(RESET)"
	cd $(ROOT)/explorer && pnpm dev

wallet: ## Start the Backpack wallet dev server
	@echo "$(CYAN)Starting Backpack wallet...$(RESET)"
	cd $(ROOT)/wallet-gui && yarn start

dapp: ## Start the DApp scaffold (localhost:3000)
	@echo "$(CYAN)Starting DApp scaffold...$(RESET)"
	cd $(ROOT)/dapp-scaffold && npm run dev

sdk: ## Build the web3.js SDK
	@echo "$(CYAN)Building web3.js SDK...$(RESET)"
	cd $(ROOT)/web3js-sdk && npm run build
	@echo "$(GREEN)✓ SDK built$(RESET)"

# ── Docker ─────────────────────────────────────────────────────────────────

docker-testnet: ## Run local testnet in Docker
	docker build -t solana-clone-testnet -f docker/Dockerfile.testnet .
	docker run -it --rm -p 8899:8899 -p 8900:8900 solana-clone-testnet

docker-explorer: ## Run explorer in Docker
	docker build -t solana-clone-explorer -f docker/Dockerfile.explorer .
	docker run -it --rm -p 3000:3000 solana-clone-explorer

# ── Utilities ──────────────────────────────────────────────────────────────

status: ## Show status of all components
	@echo ""
	@echo "  $(CYAN)Solana Clone Status$(RESET)"
	@echo "  ==================="
	@echo ""
	@[ -f $(VALIDATOR_BIN)/solana-validator ] && echo "  $(GREEN)✓$(RESET) Validator       built" || echo "  $(YELLOW)○$(RESET) Validator       not built"
	@[ -f $(VALIDATOR_BIN)/solana ] && echo "  $(GREEN)✓$(RESET) CLI             built" || echo "  $(YELLOW)○$(RESET) CLI             not built"
	@[ -f $(VALIDATOR_BIN)/solana-test-validator ] && echo "  $(GREEN)✓$(RESET) Test Validator  built" || echo "  $(YELLOW)○$(RESET) Test Validator  not built"
	@[ -d $(ROOT)/explorer/node_modules ] && echo "  $(GREEN)✓$(RESET) Explorer        installed" || echo "  $(YELLOW)○$(RESET) Explorer        not installed"
	@[ -d $(ROOT)/dapp-scaffold/node_modules ] && echo "  $(GREEN)✓$(RESET) DApp Scaffold   installed" || echo "  $(YELLOW)○$(RESET) DApp Scaffold   not installed"
	@[ -d $(ROOT)/web3js-sdk/node_modules ] && echo "  $(GREEN)✓$(RESET) Web3.js SDK     installed" || echo "  $(YELLOW)○$(RESET) Web3.js SDK     not installed"
	@[ -f $(ROOT)/.testnet.pid ] && echo "  $(GREEN)●$(RESET) Testnet         running (PID $$(cat $(ROOT)/.testnet.pid))" || echo "  $(RED)○$(RESET) Testnet         stopped"
	@echo ""

sync: ## Sync all components with upstream
	@echo "$(CYAN)Syncing with upstream repositories...$(RESET)"
	@for dir in validator web3js-sdk program-library explorer wallet-adapter wallet-gui dapp-scaffold; do \
		if [ -d $(ROOT)/$$dir/.git ]; then \
			echo "  Syncing $$dir..."; \
			cd $(ROOT)/$$dir && git fetch upstream 2>/dev/null && echo "  $(GREEN)✓$(RESET) $$dir" || echo "  $(YELLOW)○$(RESET) $$dir (no upstream)"; \
		fi \
	done

clean: ## Clean all build artifacts
	@echo "$(CYAN)Cleaning build artifacts...$(RESET)"
	cd $(ROOT)/validator && cargo clean 2>/dev/null; true
	cd $(ROOT)/program-library && cargo clean 2>/dev/null; true
	rm -rf $(ROOT)/explorer/.next
	rm -rf $(ROOT)/dapp-scaffold/.next
	rm -f $(ROOT)/.testnet.log $(ROOT)/.testnet.pid
	@echo "$(GREEN)✓ Clean$(RESET)"

all: validator programs sdk ## Build everything (validator + programs + SDK)
