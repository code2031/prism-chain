# Prism AI Tools

Four standalone Next.js apps that provide AI-powered blockchain development tools for the Prism network.

## Apps

| App | Description | Port |
|-----|-------------|------|
| **contract-auditor** | Static analysis for Rust/Anchor programs with 12 built-in security rules (missing signer, unchecked arithmetic, type cosplay, etc.). Includes a Monaco editor and severity-scored results. | 3000 |
| **explorer** | Natural language blockchain explorer. Parses queries like "balance of <address>" or "show TPS" into JSON-RPC calls and displays results. | 3000 |
| **nft-generator** | Prompt-based NFT creation. Generates Metaplex-compatible metadata and preview images across five styles (pixel-art, watercolor, 3D, anime, abstract). | 3000 |
| **portfolio-advisor** | Wallet risk analysis. Calculates concentration risk (HHI), diversification score, volatility estimate, and generates rebalancing suggestions. | 3000 |

## Quick Start

Each app runs independently:

```bash
cd ai/<app-name>
npm install
npm run dev
```

## Tech Stack

- **Framework**: Next.js 16 (App Router)
- **UI**: React 19, Tailwind CSS
- **Language**: TypeScript
- **Editor** (contract-auditor): Monaco Editor (`@monaco-editor/react`)

## Directory Structure

```
ai/
+-- contract-auditor/
|   +-- lib/auditor.ts         # 12 regex-based audit rules + scoring engine
|   +-- components/            # code-input, audit-results
|   +-- app/                   # Next.js App Router pages + API
+-- explorer/
|   +-- lib/query-parser.ts    # NL query -> RPC method mapping (10 intents)
|   +-- lib/rpc-executor.ts    # JSON-RPC call execution
|   +-- components/            # search-bar, query-result
+-- nft-generator/
|   +-- lib/metadata-builder.ts  # Metaplex metadata + SVG placeholder generation
|   +-- components/              # prompt-input, preview-panel, nft-gallery
+-- portfolio-advisor/
|   +-- lib/analyzer.ts        # HHI concentration, risk scoring, suggestions
|   +-- components/            # wallet-input, portfolio-analysis
```
