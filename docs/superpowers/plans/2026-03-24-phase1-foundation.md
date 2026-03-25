# Phase 1: Foundation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the SolClone wallet into a Phantom-quality experience and make the developer experience frictionless — faucet UI, SDK playground, Anchor templates, docs site, and performance benchmarks.

**Architecture:** Phase 1 has 3 independent workstreams that can be built in parallel: (A) Flutter wallet UX overhaul, (B) Developer experience tools (faucet, playground, templates, docs), (C) Performance baseline (benchmarks, health dashboard). Workstream A modifies `flutter-wallet/`, B creates new top-level directories (`faucet/`, `playground/`, `templates/`, `docs-site/`), and C creates `benchmarks/` and `health-dashboard/`.

**Tech Stack:** Dart/Flutter (wallet), Next.js + Tailwind (faucet, playground, health dashboard), Rust/Anchor (templates), Docusaurus (docs), TypeScript (CLI `init` command), Bash (benchmarks)

**Spec:** `docs/superpowers/specs/2026-03-24-solclone-masterplan-design.md` — Phase 1 section

---

## File Structure

### Workstream A: Flutter Wallet UX Overhaul

```
flutter-wallet/lib/
├── main.dart                              # Update: add routes for new screens, light theme
├── theme/
│   ├── app_theme.dart                     # Modify: add light theme, improve dark theme
│   └── animations.dart                    # Create: shared page transitions, spring curves
├── providers/
│   ├── wallet_provider.dart               # Modify: multi-account support, token auto-discovery
│   ├── network_provider.dart              # Existing (no changes)
│   └── accounts_provider.dart             # Create: manage multiple wallet accounts
├── services/
│   ├── rpc_service.dart                   # Modify: add getTokenAccountsByOwner, NFT fetching
│   ├── wallet_service.dart                # Modify: multi-account keypair management
│   ├── price_service.dart                 # Existing (no changes)
│   └── walletconnect_service.dart         # Create: WalletConnect v2 session management
├── screens/
│   ├── onboarding/
│   │   ├── welcome_screen.dart            # Modify: streamline to 3-tap flow
│   │   ├── create_wallet_screen.dart      # Modify: delayed backup, biometric setup
│   │   └── import_wallet_screen.dart      # Existing (no changes)
│   ├── home/
│   │   ├── home_screen.dart               # Modify: add 4th tab (Collectibles)
│   │   ├── portfolio_tab.dart             # Modify: portfolio chart, token auto-discovery
│   │   ├── activity_tab.dart              # Modify: rich icons, counterparty labels
│   │   ├── collectibles_tab.dart          # Create: NFT grid gallery
│   │   └── settings_tab.dart              # Modify: multi-account switcher, theme toggle
│   ├── send/
│   │   └── send_screen.dart               # Modify: transaction preview with human-readable breakdown
│   ├── swap/
│   │   └── swap_screen.dart               # Create: mock swap UI (real in Phase 2)
│   ├── nft/
│   │   └── nft_detail_screen.dart         # Create: full-screen NFT detail, send
│   └── walletconnect/
│       └── walletconnect_screen.dart      # Create: QR scanner, session management
├── widgets/
│   ├── portfolio_chart.dart               # Create: fl_chart portfolio donut + line chart
│   ├── nft_grid_item.dart                 # Create: NFT thumbnail card for grid
│   ├── transaction_preview.dart           # Create: human-readable tx breakdown
│   ├── account_switcher.dart              # Create: multi-account dropdown
│   └── animated_balance.dart              # Create: count-up animation for balance
│   # Existing widgets remain unchanged
└── models/
    ├── nft.dart                           # Create: NFT model (mint, name, image, collection, attributes)
    └── wallet_account.dart                # Create: multi-account model (name, pubkey, derivation path)
```

### Workstream B: Developer Experience

```
faucet/                                    # New Next.js app
├── package.json
├── next.config.ts
├── tailwind.config.ts
├── app/
│   ├── layout.tsx                         # Root layout, dark theme
│   ├── page.tsx                           # Faucet form: address input, amount selector, submit
│   └── api/
│       └── airdrop/route.ts               # POST handler: validate address, call RPC requestAirdrop, rate limit
├── components/
│   └── faucet-form.tsx                    # Client component: form with validation, loading state
└── lib/
    └── rate-limiter.ts                    # In-memory rate limiter (IP + address based)

playground/                                # New Next.js app
├── package.json
├── next.config.ts
├── app/
│   ├── layout.tsx
│   └── page.tsx                           # Monaco editor + output panel + RPC selector
├── components/
│   ├── code-editor.tsx                    # Monaco editor with TypeScript SDK types
│   ├── output-panel.tsx                   # Console output display
│   └── rpc-selector.tsx                   # Network dropdown (devnet/testnet/localnet)
└── lib/
    └── sandbox-runner.ts                  # Execute SDK code in iframe sandbox

templates/                                 # Anchor program templates
├── token/
│   ├── Anchor.toml
│   ├── Cargo.toml
│   ├── programs/token/src/lib.rs          # Mint + transfer + burn
│   └── tests/token.ts
├── nft-collection/
│   ├── programs/nft/src/lib.rs            # Create collection, mint NFTs, set metadata
│   └── tests/nft.ts
├── escrow/
│   ├── programs/escrow/src/lib.rs         # Initialize, exchange, cancel
│   └── tests/escrow.ts
├── voting/
│   ├── programs/voting/src/lib.rs         # Create proposal, vote, finalize
│   └── tests/voting.ts
└── staking-pool/
    ├── programs/staking/src/lib.rs        # Deposit, withdraw, distribute rewards
    └── tests/staking.ts

cli-wallet/src/commands/init.ts            # Create: `solclone init` scaffolds Anchor project

docs-site/                                 # Docusaurus site
├── package.json
├── docusaurus.config.js
├── sidebars.js
├── docs/
│   ├── intro.md
│   ├── getting-started.md
│   ├── cli-reference.md
│   ├── sdk-reference.md
│   ├── anchor-guide.md
│   └── network-guide.md
└── static/
    └── img/logo.svg                       # Copy from branding/
```

### Workstream C: Performance Baseline

```
benchmarks/
├── package.json
├── src/
│   ├── tps-bench.ts                       # Flood testnet with transfers, measure TPS
│   ├── latency-bench.ts                   # Measure confirmation latency (send → confirmed)
│   └── report.ts                          # Format results as markdown table
└── run.sh                                 # One-command benchmark runner

health-dashboard/                          # New Next.js app
├── package.json
├── app/
│   ├── layout.tsx
│   └── page.tsx                           # Dashboard: slot height, TPS, validators, epoch
├── components/
│   ├── stat-card.tsx                      # Single metric card
│   ├── tps-chart.tsx                      # Real-time TPS line chart
│   └── validator-table.tsx                # Validator list with stake, commission, status
└── lib/
    └── rpc-poller.ts                      # Poll RPC every 2s for cluster stats
```

---

## Tasks

### Task 1: Flutter Wallet — Light Theme + Animations Foundation

**Files:**
- Modify: `flutter-wallet/lib/theme/app_theme.dart`
- Create: `flutter-wallet/lib/theme/animations.dart`
- Modify: `flutter-wallet/lib/main.dart`

- [ ] **Step 1: Add light theme to app_theme.dart**

Add a `lightTheme` getter alongside the existing dark theme. Use white/gray surfaces, same purple/green accents. Add a `ThemeMode` toggle.

- [ ] **Step 2: Create animations.dart**

Define shared animation utilities: `SlideUpRoute` (page transition), `CountUpAnimation` (balance), `SpringCurve` (pull-to-refresh), `FadeScaleTransition` (modal).

- [ ] **Step 3: Update main.dart for theme switching**

Add `themeMode` parameter to `MaterialApp`, read from `SharedPreferences`. Wire to settings tab.

- [ ] **Step 4: Test theme switching manually**

Run: `cd flutter-wallet && flutter run -d chrome`
Verify: Toggle between dark/light in settings. All screens should render correctly in both.

- [ ] **Step 5: Commit**

```bash
git add flutter-wallet/lib/theme/ flutter-wallet/lib/main.dart
git commit -m "feat(wallet): add light theme and shared animations"
```

---

### Task 2: Flutter Wallet — Multi-Account Support

**Files:**
- Create: `flutter-wallet/lib/models/wallet_account.dart`
- Create: `flutter-wallet/lib/providers/accounts_provider.dart`
- Modify: `flutter-wallet/lib/services/wallet_service.dart`
- Create: `flutter-wallet/lib/widgets/account_switcher.dart`
- Modify: `flutter-wallet/lib/screens/home/settings_tab.dart`

- [ ] **Step 1: Create wallet_account.dart model**

```dart
class WalletAccount {
  final String name;
  final String publicKey;
  final int derivationIndex; // m/44'/501'/N'/0'
  final DateTime createdAt;
  // ... toJson, fromJson
}
```

- [ ] **Step 2: Create accounts_provider.dart**

ChangeNotifier that manages a list of `WalletAccount`s. Methods: `createAccount(name)`, `importAccount(key)`, `switchAccount(index)`, `deleteAccount(index)`. Persists to `SharedPreferences`.

- [ ] **Step 3: Update wallet_service.dart**

Support multiple keypairs by derivation index. `getKeypair(index)` derives from the master seed at `m/44'/501'/{index}'/0'`.

- [ ] **Step 4: Create account_switcher.dart widget**

Dropdown showing all accounts with name, truncated address, and balance. Tap to switch. "Add account" button at bottom.

- [ ] **Step 5: Wire into settings_tab.dart**

Add account switcher to the top of settings. Show current account with edit name option.

- [ ] **Step 6: Test account switching**

Run: `flutter run -d chrome`
Verify: Create 2 accounts, switch between them, balances update.

- [ ] **Step 7: Commit**

```bash
git add flutter-wallet/lib/models/wallet_account.dart flutter-wallet/lib/providers/accounts_provider.dart flutter-wallet/lib/services/wallet_service.dart flutter-wallet/lib/widgets/account_switcher.dart flutter-wallet/lib/screens/home/settings_tab.dart
git commit -m "feat(wallet): multi-account support with HD derivation"
```

---

### Task 3: Flutter Wallet — Token Auto-Discovery + Portfolio Chart

**Files:**
- Modify: `flutter-wallet/lib/providers/wallet_provider.dart`
- Modify: `flutter-wallet/lib/services/rpc_service.dart`
- Create: `flutter-wallet/lib/widgets/portfolio_chart.dart`
- Create: `flutter-wallet/lib/widgets/animated_balance.dart`
- Modify: `flutter-wallet/lib/screens/home/portfolio_tab.dart`

- [ ] **Step 1: Add getTokenAccountsByOwner to rpc_service.dart**

New method that calls `getTokenAccountsByOwner` RPC, parses token accounts, fetches mint info (decimals, supply). Returns list of `TokenModel`.

- [ ] **Step 2: Update wallet_provider to auto-discover tokens**

On refresh, call `getTokenAccountsByOwner` and merge results with known tokens. Show all non-zero balances automatically.

- [ ] **Step 3: Create portfolio_chart.dart**

`fl_chart` donut chart showing token allocation by USD value. Tap a segment to highlight. Below it, a 7-day portfolio line chart.

- [ ] **Step 4: Create animated_balance.dart**

Widget that animates from old balance to new balance using a count-up effect over 500ms.

- [ ] **Step 5: Update portfolio_tab.dart**

Replace static balance display with `AnimatedBalance`. Add `PortfolioChart` above the token list. Pull-to-refresh triggers token auto-discovery.

- [ ] **Step 6: Test with multiple tokens**

Start local testnet, create 3 SPL tokens via CLI, fund wallet. Verify all appear in portfolio with chart.

- [ ] **Step 7: Commit**

```bash
git add flutter-wallet/lib/
git commit -m "feat(wallet): token auto-discovery and portfolio chart"
```

---

### Task 4: Flutter Wallet — NFT Collectibles Tab

**Files:**
- Create: `flutter-wallet/lib/models/nft.dart`
- Create: `flutter-wallet/lib/screens/home/collectibles_tab.dart`
- Create: `flutter-wallet/lib/screens/nft/nft_detail_screen.dart`
- Create: `flutter-wallet/lib/widgets/nft_grid_item.dart`
- Modify: `flutter-wallet/lib/screens/home/home_screen.dart`

- [ ] **Step 1: Create nft.dart model**

```dart
class NftModel {
  final String mint;
  final String name;
  final String imageUrl;
  final String? collection;
  final Map<String, String> attributes;
}
```

- [ ] **Step 2: Create nft_grid_item.dart**

Card widget: rounded image thumbnail, name below, collection badge. Tap navigates to detail.

- [ ] **Step 3: Create collectibles_tab.dart**

GridView of `NftGridItem`s. Empty state: "No collectibles yet" with illustration. Pull-to-refresh. Fetches NFTs by querying token accounts where amount=1 and decimals=0.

- [ ] **Step 4: Create nft_detail_screen.dart**

Full-screen image, name, collection, attributes list, mint address (copyable), "Send" button.

- [ ] **Step 5: Add Collectibles tab to home_screen.dart**

Change bottom nav from 3 tabs to 4: Portfolio, Collectibles, Activity, Settings.

- [ ] **Step 6: Test NFT display**

Create an NFT via `./examples/create-nft.sh`, verify it appears in Collectibles tab.

- [ ] **Step 7: Commit**

```bash
git add flutter-wallet/lib/
git commit -m "feat(wallet): NFT collectibles tab with grid gallery"
```

---

### Task 5: Flutter Wallet — Transaction Preview + Improved Send

**Files:**
- Create: `flutter-wallet/lib/widgets/transaction_preview.dart`
- Modify: `flutter-wallet/lib/screens/send/send_screen.dart`

- [ ] **Step 1: Create transaction_preview.dart**

Bottom sheet widget showing:
- Human-readable action: "Send 5.2 SCLONE"
- Recipient: truncated address with copy button
- Network fee: "~0.000005 SCLONE"
- Total deducted: "5.200005 SCLONE"
- "Confirm" gradient button + "Cancel" text button

- [ ] **Step 2: Update send_screen.dart**

After entering amount and recipient, show `TransactionPreview` bottom sheet instead of going straight to confirmation. Only submit transaction after user taps "Confirm" in preview.

- [ ] **Step 3: Test send flow**

Run wallet, send SOL to another address. Verify preview sheet shows, confirm sends, cancel aborts.

- [ ] **Step 4: Commit**

```bash
git add flutter-wallet/lib/widgets/transaction_preview.dart flutter-wallet/lib/screens/send/send_screen.dart
git commit -m "feat(wallet): transaction preview before sending"
```

---

### Task 6: Flutter Wallet — Mock Swap Screen

**Files:**
- Create: `flutter-wallet/lib/screens/swap/swap_screen.dart`
- Modify: `flutter-wallet/lib/screens/home/portfolio_tab.dart`

- [ ] **Step 1: Create swap_screen.dart**

Swap interface: "From" token selector + amount, "To" token selector + estimated output, swap arrow button in middle, slippage setting, "Swap" button. For now, show a "Coming in Phase 2" dialog on tap.

- [ ] **Step 2: Add swap button to portfolio_tab.dart**

Add a "Swap" action button next to Send/Receive in the quick actions row.

- [ ] **Step 3: Test navigation**

Verify swap screen opens, token selectors work, swap button shows coming-soon message.

- [ ] **Step 4: Commit**

```bash
git add flutter-wallet/lib/screens/swap/ flutter-wallet/lib/screens/home/portfolio_tab.dart
git commit -m "feat(wallet): mock swap screen (Phase 2 placeholder)"
```

---

### Task 7: Faucet Web UI

**Files:**
- Create: `faucet/package.json`
- Create: `faucet/next.config.ts`
- Create: `faucet/tailwind.config.ts`
- Create: `faucet/postcss.config.js`
- Create: `faucet/app/layout.tsx`
- Create: `faucet/app/page.tsx`
- Create: `faucet/app/api/airdrop/route.ts`
- Create: `faucet/components/faucet-form.tsx`
- Create: `faucet/lib/rate-limiter.ts`
- Create: `faucet/app/globals.css`

- [ ] **Step 1: Scaffold Next.js app**

```bash
cd /home/pranav/solana-clone
npx create-next-app@latest faucet --typescript --tailwind --app --src-dir=false --import-alias="@/*" --no-eslint
```

- [ ] **Step 2: Create rate-limiter.ts**

In-memory map: `{key: {count, lastReset}}`. Max 10 requests per hour per IP. Max 5 per hour per wallet address. `checkLimit(ip, address): boolean`.

- [ ] **Step 3: Create the API route (app/api/airdrop/route.ts)**

POST handler: validate Base58 address (regex), check rate limit, call `fetch(RPC_URL, { method: 'requestAirdrop', params: [address, lamports] })`, return signature or error.

- [ ] **Step 4: Create faucet-form.tsx**

Client component: address input (with paste button), amount selector (1/2/5 SOL radio buttons), submit button with loading spinner, success/error toast, transaction signature link.

- [ ] **Step 5: Create page.tsx and layout.tsx**

Dark theme page with SolClone branding, network selector (devnet/testnet), the faucet form centered, and a "powered by SolClone" footer.

- [ ] **Step 6: Test locally**

```bash
cd faucet && npm run dev
# Open http://localhost:3001
# Enter an address, request airdrop, verify success
```

- [ ] **Step 7: Add to Makefile**

Add `faucet:` target: `cd $(ROOT)/faucet && npm run dev`

- [ ] **Step 8: Commit**

```bash
git add faucet/ Makefile
git commit -m "feat: add faucet web UI with rate limiting"
```

---

### Task 8: SDK Playground

**Files:**
- Create: `playground/` (Next.js app with Monaco editor)

- [ ] **Step 1: Scaffold Next.js app**

```bash
npx create-next-app@latest playground --typescript --tailwind --app --no-eslint
cd playground && npm install @monaco-editor/react
```

- [ ] **Step 2: Create code-editor.tsx**

Monaco editor component with TypeScript language, pre-loaded with SolClone SDK type definitions. Default code: `// Connect to SolClone devnet and check balance`.

- [ ] **Step 3: Create output-panel.tsx**

Console-style output panel below the editor. Captures `console.log` from sandboxed execution. Shows results, errors, timing.

- [ ] **Step 4: Create rpc-selector.tsx**

Dropdown: Devnet (localhost:8899), Testnet (localhost:8799), Custom URL. Passes selected URL to the sandbox runner.

- [ ] **Step 5: Create sandbox-runner.ts**

Executes code in an iframe sandbox. Injects a mock `solana` object that makes real RPC calls to the selected network. Returns output to the parent.

- [ ] **Step 6: Create page.tsx**

Split-pane layout: editor on left (70%), output on right (30%). Top bar: "SolClone Playground" + network selector + "Run" button. Pre-loaded examples dropdown.

- [ ] **Step 7: Test with a real RPC call**

Start local testnet, open playground, run `getBalance` example. Verify output shows balance.

- [ ] **Step 8: Commit**

```bash
git add playground/
git commit -m "feat: add SDK playground with Monaco editor and live RPC"
```

---

### Task 9: Anchor Program Templates

**Files:**
- Create: `templates/token/` — full Anchor project
- Create: `templates/nft-collection/`
- Create: `templates/escrow/`
- Create: `templates/voting/`
- Create: `templates/staking-pool/`

- [ ] **Step 1: Create token template**

Anchor program with instructions: `initialize_mint`, `mint_to`, `transfer`, `burn`. Test file that deploys and exercises all instructions.

- [ ] **Step 2: Create NFT collection template**

Anchor program: `create_collection`, `mint_nft` (with metadata), `transfer_nft`. Test file.

- [ ] **Step 3: Create escrow template**

Anchor program: `initialize_escrow` (lock token A), `exchange` (swap A for B), `cancel` (refund). Test file.

- [ ] **Step 4: Create voting template**

Anchor program: `create_proposal`, `cast_vote`, `finalize` (count votes, declare result). Test file.

- [ ] **Step 5: Create staking pool template**

Anchor program: `initialize_pool`, `deposit`, `withdraw`, `distribute_rewards`. Test file.

- [ ] **Step 6: Verify each template builds**

```bash
for dir in templates/*/; do
  echo "Building $dir..."
  cd "$dir" && anchor build && cd ../..
done
```

- [ ] **Step 7: Commit**

```bash
git add templates/
git commit -m "feat: add 5 Anchor program templates (token, NFT, escrow, voting, staking)"
```

---

### Task 10: `solclone init` CLI Command

**Files:**
- Create: `cli-wallet/src/commands/init.ts`
- Modify: `cli-wallet/src/index.ts`

- [ ] **Step 1: Create init.ts**

Command: `solclone init <project-name> [--template token|nft|escrow|voting|staking]`. Copies the selected template from `templates/` to the target directory. Updates `Anchor.toml` with SolClone network URLs. Runs `npm install` if package.json exists.

- [ ] **Step 2: Register in index.ts**

Import and register the `init` command in the main CLI entry point.

- [ ] **Step 3: Build and test**

```bash
cd cli-wallet && npm run build
npx solclone init my-token-project --template token
ls my-token-project/  # Verify files copied
```

- [ ] **Step 4: Commit**

```bash
git add cli-wallet/src/commands/init.ts cli-wallet/src/index.ts
git commit -m "feat(cli): add solclone init command for project scaffolding"
```

---

### Task 11: Docs Site (Docusaurus)

**Files:**
- Create: `docs-site/` — full Docusaurus project

- [ ] **Step 1: Scaffold Docusaurus**

```bash
npx create-docusaurus@latest docs-site classic --typescript
```

- [ ] **Step 2: Configure branding**

Update `docusaurus.config.js`: title "SolClone Docs", tagline, logo from `branding/logo.svg`, dark theme default, navbar links (Docs, API, Tutorials, GitHub).

- [ ] **Step 3: Write core docs**

Migrate and adapt content from existing `docs/` directory:
- `docs/intro.md` — What is SolClone, architecture overview
- `docs/getting-started.md` — Install, build validator, start testnet, first transaction
- `docs/cli-reference.md` — All `solclone` CLI commands
- `docs/sdk-reference.md` — Web3.js SDK methods with examples
- `docs/anchor-guide.md` — Building programs with Anchor on SolClone
- `docs/network-guide.md` — Devnet, testnet, mainnet configs and usage

- [ ] **Step 4: Test locally**

```bash
cd docs-site && npm start
# Verify at http://localhost:3000
```

- [ ] **Step 5: Add to Makefile**

Add `docs:` target: `cd $(ROOT)/docs-site && npm start`

- [ ] **Step 6: Commit**

```bash
git add docs-site/ Makefile
git commit -m "feat: add Docusaurus docs site with core documentation"
```

---

### Task 12: Benchmark Suite

**Files:**
- Create: `benchmarks/package.json`
- Create: `benchmarks/src/tps-bench.ts`
- Create: `benchmarks/src/latency-bench.ts`
- Create: `benchmarks/src/report.ts`
- Create: `benchmarks/run.sh`

- [ ] **Step 1: Create package.json**

Dependencies: `@solana/web3.js`, `tsx`. Scripts: `bench:tps`, `bench:latency`, `bench:all`.

- [ ] **Step 2: Create tps-bench.ts**

Generate 1000 pre-signed transfer transactions. Submit all via `sendTransaction` in parallel batches of 100. Measure wall-clock time from first send to last confirmation. Calculate TPS = confirmed / elapsed.

- [ ] **Step 3: Create latency-bench.ts**

Send 100 individual transactions sequentially. For each, measure time from `sendTransaction` to `confirmTransaction`. Report: min, max, median, p95, p99 latency.

- [ ] **Step 4: Create report.ts**

Format benchmark results as a markdown table. Write to `benchmarks/RESULTS.md`.

- [ ] **Step 5: Create run.sh**

```bash
#!/usr/bin/env bash
echo "Starting local testnet..."
# Start testnet, run benchmarks, stop testnet, print report
```

- [ ] **Step 6: Run benchmarks**

```bash
cd benchmarks && bash run.sh
cat RESULTS.md
```

- [ ] **Step 7: Commit**

```bash
git add benchmarks/
git commit -m "feat: add TPS and latency benchmark suite"
```

---

### Task 13: Network Health Dashboard

**Files:**
- Create: `health-dashboard/` (Next.js app)

- [ ] **Step 1: Scaffold Next.js app**

```bash
npx create-next-app@latest health-dashboard --typescript --tailwind --app --no-eslint
cd health-dashboard && npm install recharts swr
```

- [ ] **Step 2: Create rpc-poller.ts**

SWR fetcher that calls `getEpochInfo`, `getRecentPerformanceSamples`, `getVoteAccounts`, `getBlockHeight` every 2 seconds.

- [ ] **Step 3: Create stat-card.tsx**

Card component: label, large value, trend indicator (up/down arrow + percentage). Used for slot height, TPS, validator count, epoch progress.

- [ ] **Step 4: Create tps-chart.tsx**

`recharts` line chart showing TPS over last 60 samples. Auto-scrolling.

- [ ] **Step 5: Create validator-table.tsx**

Table: validator identity (truncated), stake, commission, last vote, status (active/delinquent).

- [ ] **Step 6: Create page.tsx**

Grid layout: 4 stat cards on top (Slot, TPS, Validators, Epoch), TPS chart in middle, validator table at bottom. Auto-refresh, dark theme, SolClone branding.

- [ ] **Step 7: Test with local testnet**

Start testnet, run dashboard. Verify stats update in real-time.

- [ ] **Step 8: Add to Makefile**

Add `health:` target.

- [ ] **Step 9: Commit**

```bash
git add health-dashboard/ Makefile
git commit -m "feat: add network health dashboard with real-time stats"
```

---

## Summary

| Task | Component | Workstream |
|------|-----------|-----------|
| 1 | Light theme + animations | A: Wallet |
| 2 | Multi-account support | A: Wallet |
| 3 | Token auto-discovery + portfolio chart | A: Wallet |
| 4 | NFT collectibles tab | A: Wallet |
| 5 | Transaction preview | A: Wallet |
| 6 | Mock swap screen | A: Wallet |
| 7 | Faucet web UI | B: Dev Experience |
| 8 | SDK playground | B: Dev Experience |
| 9 | Anchor templates (5 programs) | B: Dev Experience |
| 10 | `solclone init` CLI | B: Dev Experience |
| 11 | Docs site (Docusaurus) | B: Dev Experience |
| 12 | Benchmark suite | C: Performance |
| 13 | Health dashboard | C: Performance |

**Parallel execution:** Tasks 1-6 (Wallet), 7-11 (DevEx), and 12-13 (Performance) are independent workstreams. Within each workstream, tasks are sequential.
