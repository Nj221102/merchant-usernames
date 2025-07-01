# BIP353 Username API - Introduction

## Overview

TypeScript API for creating BIP353-compliant usernames with Lightning Network BOLT12 offers. This service enables Lightning Address-style identifiers (username@domain.com) through automated DNS TXT record creation.

## Core Features

### TypeScript Architecture

- Full type safety with strict TypeScript configuration
- Type-safe configuration management and validation
- Comprehensive error handling with structured responses

### Status Tracking System

- `GET /api/status/:requestId` endpoint for job monitoring
- UUID-based request tracking with Redis storage
- Real-time status updates: `pending` → `processing` → `completed`/`failed`
- Persistent status storage with configurable TTL

### Production-Ready Features

- Redis-backed queue system for scalable processing
- Background worker for DNS record creation
- Cloudflare API integration for DNS management
- Security middleware (IP whitelisting, API key authentication)
- Comprehensive logging with Winston
- Input validation with Joi schemas

## API Endpoints

### Create Username

**`POST /api/username`**

- Queues username creation request
- Returns unique `requestId` for status tracking
- Creates DNS TXT record with BIP353-compliant format

### Check Status

**`GET /api/status/:requestId`**

- Real-time job status monitoring
- Returns current processing state and results
- Persistent status tracking via Redis

## System Architecture

```
Client Request
     ↓
API Server (Express + TypeScript)
     ↓
Redis Queue
     ↓
Background Worker
     ↓
Cloudflare API
     ↓
DNS TXT Record Created
```

## Live Testing

The system has been verified with end-to-end testing:

```bash
# Health Check
curl http://localhost:3000/health

# Username Creation
curl -X POST http://localhost:3000/api/username \
  -H "Authorization: Bearer test_api_key_123" \
  -d '{"username": "alice", "offer": "lno1pqq..."}'

# Status Tracking
curl -X GET http://localhost:3000/api/status/REQUEST_ID \
  -H "Authorization: Bearer test_api_key_123"

```

## Key Benefits

- **Production Ready**: Clean, professional codebase with comprehensive error handling
- **Reliability**: Asynchronous processing with configurable retry logic and proper error handling
- **Scalability**: Redis-backed queue system for horizontal scaling
- **Security**: Multi-layer security with API key authentication and IP whitelisting
- **Code Quality**: TypeScript strict mode, Prettier formatting, and modern ES2022 configuration
- **Monitoring**: Health checks, real-time status tracking, and structured logging with Winston
- **Standards Compliance**: Full BIP353 specification adherence with proper DNS TXT record format
- **Developer Experience**: Hot reload development environment, clean build system, and comprehensive tooling
