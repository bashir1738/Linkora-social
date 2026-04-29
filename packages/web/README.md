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

The app requires three environment variables to connect to a Soroban network.

Copy the example file and fill in your values:

```bash
cp .env.example .env.local
```

| Variable | Description |
|---|---|
| `NEXT_PUBLIC_SOROBAN_RPC_URL` | Soroban RPC endpoint for the target network |
| `NEXT_PUBLIC_NETWORK_PASSPHRASE` | Network passphrase (identifies Testnet, Mainnet, etc.) |
| `NEXT_PUBLIC_CONTRACT_ID` | Deployed Linkora contract ID on the target network |

The `.env.example` file is pre-filled for Testnet. For other networks:

- **Mainnet**: `https://soroban-mainnet.stellar.org` / `Public Global Stellar Network ; September 2015`
- **Local sandbox**: `http://localhost:8000/soroban/rpc` / `Standalone Network ; February 2017`

The app will throw a clear error at startup if any variable is missing. `.env*.local` files are gitignored and should never be committed.

## Notes

- This scaffold intentionally keeps the first page minimal.
- Contract code and existing contract workspace remain unchanged.
