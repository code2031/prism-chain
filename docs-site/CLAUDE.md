# CLAUDE.md -- SolClone Docs Site

## Overview

Docusaurus-powered documentation site for the SolClone project. Contains guides, references, and tutorials for developers, validators, and users.

## Quick Start

```bash
cd docs-site
npm install
npm start          # Dev server on http://localhost:3000
npm run build      # Production build to build/
```

## Documentation Pages

| Doc | Path | Description |
|-----|------|-------------|
| **Intro** | `docs/intro.md` | Project overview and goals |
| **Getting Started** | `docs/getting-started.md` | Installation and first steps |
| **CLI Reference** | `docs/cli-reference.md` | All CLI wallet commands |
| **SDK Guide** | `docs/sdk-guide.md` | Using @solclone/web3.js |
| **Anchor Guide** | `docs/anchor-guide.md` | Building programs with Anchor |
| **DeFi Guide** | `docs/defi-guide.md` | Interacting with SolSwap, SolLend, SCUSD |
| **NFT Guide** | `docs/nft-guide.md` | Minting and trading NFTs on SolClone |

## Key Files

- `docusaurus.config.ts` -- Site configuration (title, URL, navbar, footer)
- `sidebars.ts` -- Sidebar navigation structure
- `docs/` -- Markdown documentation files
- `blog/` -- Blog posts (release notes, announcements)
- `src/` -- Custom React components and pages
- `static/` -- Static assets (images, favicons)

## Structure

```
docs-site/
+-- docs/                    # Documentation markdown files
+-- blog/                    # Blog posts
+-- src/                     # Custom pages and components
+-- static/                  # Static assets
+-- docusaurus.config.ts     # Site configuration
+-- sidebars.ts              # Navigation sidebar config
+-- package.json
+-- tsconfig.json
```

## Editing Docs

Add or edit Markdown files in `docs/`. The sidebar auto-generates from `sidebars.ts`. Use standard Docusaurus features: admonitions, code blocks, tabs, MDX components.
