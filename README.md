# Merchant Usernames - Backend Services

## Project Synopsis

This project aims to create a non-custodial payment alternative to the Cashu payment system that Shopstr currently uses. This will allow users to use Greenlight nodes to create BOLT 12 offers linked to "BIP 353" usernames like "user@shopstr.store", which users can use to receive payment directly without a custodial intermediary.

## Repository Structure

This repository contains the backend services for the merchant usernames project:

### 📁 `/dns-api/`
DNS API service for handling BIP 353 username resolution and management.

**Technology Stack:**
- Node.js/TypeScript
- Cloudflare integration for DNS management
- Queue-based processing system

### 📁 `/greenlight-backend/`
Production-ready Rust service for Lightning wallets with Bolt12 support.

**Technology Stack:**
- Rust
- Lightning Network integration
- Greenlight node management
- BOLT 12 offer generation and management
- WebSocket support for real-time updates

## Key Features

- **Non-custodial payments**: Users maintain full control over their funds
- **BIP 353 usernames**: Human-readable payment addresses (e.g., `user@shopstr.store`)
- **BOLT 12 offers**: Modern Lightning Network payment protocol
- **Greenlight integration**: Reliable Lightning node infrastructure
- **DNS resolution**: Seamless username to payment offer mapping

## Getting Started

Each service has its own setup instructions:

- See `/dns-api/README.md` for DNS API setup
- See `/greenlight-backend/README.md` for Lightning backend setup

## Architecture

The system works by:

1. Users create Lightning nodes via the Greenlight backend
2. BOLT 12 offers are generated for payment reception
3. DNS API maps BIP 353 usernames to Lightning payment offers
4. Shopstr users can send payments directly to human-readable addresses
5. Recipients receive payments non-custodially through their Lightning nodes

## License

See individual service directories for licensing information.
