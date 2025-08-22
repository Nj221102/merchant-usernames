# Greenlight Lightning Wallet Backend

A production-ready Rust backend service for Bitcoin Lightning wallets using **Blockstream Greenlight**.

## Overview

This backend provides:
- User authentication with encrypted seed storage
- Lightning node management via Greenlight
- Bolt12 offer creation on Bitcoin mainnet
- REST API endpoints for wallet operations

## Technology

- **Language**: Rust
- **Framework**: Axum (async web server)
- **Database**: PostgreSQL
- **Lightning**: Blockstream Greenlight
- **Network**: Bitcoin Mainnet

## Getting Started

See [SETUP.md](SETUP.md) for complete setup instructions and testing guide.

## API Endpoints

- `POST /auth/register` - Register user
- `POST /auth/login` - User login
- `GET /health` - Health check
- `POST /node/register` - Register Lightning node
- `POST /node/recover` - Recover Lightning node
- `GET /node/info` - Get node information
- `GET /node/balance` - Get node balance
- `POST /node/offer` - Create Bolt12 offer

## License

MIT License