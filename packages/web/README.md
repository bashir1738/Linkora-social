# Web Package

This package bootstraps the Linkora web frontend using Next.js App Router and TypeScript.

## Prerequisites

- Node.js 18+
- pnpm 9+

## Install workspace dependencies

From repository root:

```bash
pnpm install
```

## Run the web app

From repository root:

```bash
pnpm --filter web dev
```

Or from this directory:

```bash
pnpm dev
```

## Build and lint

From repository root:

```bash
pnpm --filter web build
pnpm --filter web lint
```

## Environment Setup

Copy the example file and fill in your values:

```bash
cp .env.example .env.local
```

| Variable | Description |
|---|---|
| `NEXT_PUBLIC_SOROBAN_RPC_URL` | Soroban RPC endpoint (e.g. `https://soroban-testnet.stellar.org`) |
| `NEXT_PUBLIC_NETWORK_PASSPHRASE` | Stellar network passphrase |
| `NEXT_PUBLIC_CONTRACT_ID` | Deployed Linkora contract ID |

The app will throw an error at startup if any of these variables are missing.

`.env.example` is pre-filled for Testnet. For local sandbox or Mainnet, update the values accordingly. Never commit `.env.local` or any `.env*.local` file.

## End-to-End Testing with Playwright

### Setup

Playwright is installed as a dev dependency. To install browsers:

```bash
pnpm install
pnpm exec playwright install
```

### Running E2E Tests

From repository root:

```bash
# Run all E2E tests
pnpm --filter web test:e2e

# Run tests in UI mode (interactive)
pnpm --filter web test:e2e:ui

# Run tests in headed mode (see browser)
pnpm --filter web test:e2e:headed

# Debug mode with inspector
pnpm --filter web test:e2e:debug
```

From this directory:

```bash
pnpm test:e2e
pnpm test:e2e:ui
pnpm test:e2e:headed
pnpm test:e2e:debug
```

### Test Coverage

The E2E tests verify critical user flows:

1. **Wallet Connection & Profile Registration** (`tests/e2e/wallet-profile.spec.ts`)
   - Connect wallet → register profile → verify profile page shows username
   - Disconnect wallet functionality
   - Address display in header

2. **Post Creation & Feed** (`tests/e2e/posts.spec.ts`)
   - Create post → verify post appears on the feed
   - Feed loads with existing posts
   - Post detail navigation
   - Post metadata (author, timestamp)

3. **Post Tipping** (`tests/e2e/tipping.spec.ts`)
   - Tip a post → verify tip total updates on the post detail page
   - Tip button visibility
   - Tip amount display

### Configuration

The Playwright configuration is in `playwright.config.ts`:

- **Base URL**: `http://localhost:3000`
- **Test Directory**: `tests/e2e`
- **Browsers**: Chromium, Firefox, WebKit, Mobile Chrome, Mobile Safari
- **Screenshots**: Captured on failure
- **Traces**: Recorded on first retry for debugging
- **Dev Server**: Automatically starts `pnpm dev` before tests (can be disabled with `CI` env var)

### Requirements

Tests require a running local Stellar sandbox with deployed contracts. Use the integration test infrastructure:

```bash
# From repository root
bash tests/integration/run_e2e.sh
```

This will:
1. Start a local Stellar sandbox
2. Deploy contracts
3. Fund test identities

Then run Playwright tests in another terminal.

### CI/CD Integration

E2E tests can be run in CI workflows. Set `CI=true` environment variable to:
- Skip server reuse (always start fresh)
- Retry failed tests up to 2 times
- Use single worker (no parallelization)

Example CI configuration:

```yaml
- name: Run E2E Tests
  env:
    CI: true
  run: pnpm --filter web test:e2e
```

### Debugging

Use Playwright Inspector for interactive debugging:

```bash
pnpm test:e2e:debug
```

Or view test reports:

```bash
pnpm exec playwright show-report
```

## Notes

- This scaffold intentionally keeps the first page minimal.
- Contract code and existing contract workspace remain unchanged.
