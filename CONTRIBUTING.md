# Contributing to Stellar Escrow Toolkit

Thank you for contributing to [Stellar Escrow Toolkit](https://github.com/stellar-escrow-toolkit/stellar-escrow-toolkit). The project provides reusable Soroban escrow contracts, a TypeScript SDK, React components, and CLI tooling for conditional payments on Stellar.

Because the contracts can hold real value, correctness and operational safety take priority over feature speed. Please read this guide and [`SECURITY.md`](./SECURITY.md) before opening a pull request.

## Code of conduct and project values

Contributors are expected to:

- communicate respectfully and assume good intent;
- prefer clear, reviewable changes over clever abstractions;
- document assumptions and user-visible behavior;
- treat contract authorization, token accounting, and state transitions as security-sensitive;
- never include private keys, credentials, customer data, or live transaction identifiers in commits, fixtures, or issues.

## Development setup

### Prerequisites

- Node.js 20 or newer (Node.js 22 is used in CI)
- npm 10 or newer
- Rust stable
- the Soroban WebAssembly target:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### Clone and install

```bash
git clone https://github.com/stellar-escrow-toolkit/stellar-escrow-toolkit.git
cd stellar-escrow-toolkit
npm install
```

### Validate the workspace

Run the project checks locally:

```bash
# Format Rust contracts without changing files
cd contracts && cargo fmt --all -- --check && cd ..

# Compile and run all Soroban contract tests
make contracts-test

# Build the deployable contract artifacts
make contracts-build

# Build all TypeScript packages
npm run build

# Run TypeScript package tests
npm test
```

`npm test` currently covers the SDK test suite. The React and CLI packages are validated through the workspace build and type checks.

## Repository structure

```text
contracts/
  escrow-core/       Two-party and arbiter escrow with timelock refunds
  escrow-milestone/  Milestone submissions, approvals, releases, and disputes
  escrow-multisig/   M-of-N proposal approval and treasury actions
packages/
  sdk/               @stellar-escrow/sdk contract clients and event watcher
  react/             @stellar-escrow/react hooks and UI components
  cli/               stellar-escrow inspection CLI
examples/             Example integrations
docs/                 Architecture and contract-model documentation
```

Keep chain-specific behavior in `contracts/`, shared client behavior in the SDK, and presentation-only behavior in `packages/react`. Avoid coupling contract crates to frontend or CLI code.

## Contract development rules

Every contract change should be reviewed as a state-machine change. Before implementation, write down:

1. the states and allowed transitions;
2. who can authorize each transition;
3. every token transfer and its recipient;
4. the behavior at deadlines, duplicate calls, and failed transfers;
5. the accounting invariant for funds held by the contract.

Contract pull requests should:

- use explicit `Result` errors for expected validation failures;
- validate authorization before performing state-changing work;
- reject non-positive amounts and invalid deadlines;
- prevent terminal-state replay and duplicate identifiers;
- test token balances before and after every settlement path;
- consider Soroban storage TTL for any state that may outlive the default lifetime;
- emit stable, documented events when indexers depend on them.

Do not add fees, AMM swaps, insurance, registries, or application-specific proof logic to a base escrow primitive without documenting the new accounting and authorization boundaries.

## SDK, React, and CLI changes

When a contract method or serialized type changes, update the corresponding SDK client and types in the same pull request. Add tests for:

- argument encoding and method names;
- numeric values that require `bigint`;
- enum and optional-value decoding;
- invalid hashes, addresses, IDs, and network configuration;
- RPC simulation failures and event pagination/retry behavior.

React changes should keep hooks usable without a wallet provider and should cover loading, empty, error, and stale-request states. CLI changes should provide a useful non-zero exit code on RPC or decoding failures and should not print secrets.

## Branches and commits

Create a focused branch from `main`:

```bash
git switch main
git pull --ff-only origin main
git switch -c feat/short-description
```

Use focused commits with [Conventional Commits](https://www.conventionalcommits.org/) prefixes, for example:

```text
feat(contracts): add vesting escrow extension
fix(sdk): decode milestone proof hashes
test(multisig): cover duplicate signer approvals
docs: explain escrow state transitions
```

Do not mix unrelated formatting, dependency upgrades, or generated artifacts into a feature change.

## Pull requests

A pull request should explain:

- what changed and why;
- the affected contracts, packages, or public APIs;
- the state transitions and accounting impact;
- migration or compatibility concerns;
- how the change was tested;
- any remaining limitation or testnet-only assumption.

Before requesting review, confirm:

- [ ] Rust formatting passes;
- [ ] contract tests pass;
- [ ] TypeScript packages build;
- [ ] TypeScript tests pass;
- [ ] public API and README documentation are updated;
- [ ] no secrets or live transaction data are included;
- [ ] the change has an appropriate security impact assessment.

Keep pull requests small enough to review. New contract features should include focused tests for authorization, duplicate calls, deadline behavior, token conservation, and failure/retry paths.

## Reporting security issues

Do not open a public issue for a vulnerability that could expose funds, keys, credentials, or private user data. Follow the private-reporting guidance in [`SECURITY.md`](./SECURITY.md).

