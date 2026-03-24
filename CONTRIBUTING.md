# Contributing to SolClone

## Development Setup

1. Fork and clone this repo
2. Run `./scripts/setup.sh --all` to install everything
3. Start a local testnet: `make testnet-bg`
4. Make your changes
5. Test against the local testnet
6. Submit a PR

## Component Guidelines

### Validator (Rust)
- Follow Rust conventions (`cargo fmt`, `cargo clippy`)
- Add tests for new functionality
- Build with `make validator`

### CLI Wallet (TypeScript)
- Follow existing patterns in `cli-wallet/src/`
- Add new commands in `src/commands/`
- Build with `cd cli-wallet && npm run build`

### Flutter Wallet (Dart)
- Follow Flutter/Dart conventions (`dart format`)
- Use Provider for state management
- Test on multiple platforms

### Explorer / DApp Scaffold (Next.js)
- Follow existing Next.js patterns
- Test with `pnpm test` / `npm test`

## Networks

Test against all networks before submitting:
- `make testnet` (localnet)
- `docker compose --profile devnet up` (devnet)
- `docker compose --profile testnet up` (testnet)

## Commit Messages

Use conventional commits:
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation
- `refactor:` code restructuring
- `test:` adding tests
- `chore:` maintenance
