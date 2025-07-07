# BIP353 Merchant Username API

A production-ready TypeScript API for creating BIP353-compliant merchant usernames with Lightning Network BOLT12 offers, featuring async processing, status tracking, and comprehensive error handling.

## Overview

Creates DNS TXT records via Cloudflare API for Lightning Address-style identifiers (username@domain.com) that comply with BIP353 standard.

**Architecture**: Client → API Server → Redis Queue → Worker → Cloudflare API

## Features

- **TypeScript** - Full type safety with strict configuration (ES2022)
- **Async Processing** - Redis queue for scalable background operations
- **Status Tracking** - Real-time job monitoring with UUID request IDs
- **Security** - IP whitelisting, API key authentication, and input validation
- **Logging** - logging with Winston (structured JSON)
- **Error Handling** - Comprehensive error responses with proper HTTP status codes
- **Code Quality** - Prettier formatting, TypeScript strict mode, and validation
- **Development Tools** - Hot reload with ts-node-dev for both server and worker

## Project Structure

```
merchant-usernames/
├── package.json              # Dependencies and scripts
├── tsconfig.json             # TypeScript configuration (ES2022)
├── .env                      # Environment variables (not in git)
├── .env.example              # Environment template
├── .prettierrc               # Code formatting configuration
├── .prettierignore           # Files to skip formatting
├── .gitignore                # Git ignore patterns
├── INTRODUCTION.md           # Project overview and features
├── README.md                 # Setup and usage guide
├── dist/                     # Compiled JavaScript (auto-generated)
├── logs/                     # Application logs (auto-created)
│   ├── combined.log          # All logs
│   └── error.log             # Error logs only
└── src/
    ├── server.ts             # Main API server
    ├── worker.ts             # Background worker process
    ├── types/
    │   └── index.ts          # TypeScript type definitions
    ├── config/
    │   └── index.ts          # Configuration management
    ├── middleware/
    │   ├── auth.ts           # Authentication & IP whitelisting
    │   └── validation.ts     # Request validation middleware
    ├── services/
    │   ├── cloudflare.ts     # Cloudflare DNS API integration
    │   └── queue.ts          # Redis queue management
    ├── utils/
    │   └── logger.ts         # Winston logging configuration
    ├── validation/
    │   └── schemas.ts        # Joi validation schemas
    └── routes/
        └── api.ts            # API route definitions
```

## Setup

1. **Install dependencies**

   ```bash
   npm install
   ```

2. **Build project**

   ```bash
   npm run build
   ```

   For clean builds:

   ```bash
   npm run clean && npm run build
   ```

3. **Configure environment**

   ```bash
   cp .env.example .env
   ```

   Edit `.env` with your settings:

   ```bash
   # Server Configuration
   PORT=3000
   NODE_ENV=development
   DOMAIN=yourdomain.com

   # Redis Configuration
   REDIS_URL=redis://localhost:6379
   REDIS_QUEUE_NAME=bip353_username_queue

   # Cloudflare API Configuration
   CLOUDFLARE_API_TOKEN=your_cloudflare_api_token_here
   CLOUDFLARE_ZONE_ID=your_cloudflare_zone_id_here

   # Security Configuration
   API_KEYS=test_api_key_123,production_key_456
   WHITELISTED_IPS=127.0.0.1,::1

   # Worker Configuration
   WORKER_MAX_RETRIES=3
   WORKER_RETRY_DELAY_MS=1000
   ```

4. **Setup Redis**

   ```bash
   # Using Docker (recommended)
   docker run -d --name redis -p 6379:6379 redis:7-alpine
   ```

5. **Start services**

   ```bash
   # Development (2 terminals)
   npm run dev        # API server
   npm run dev:worker # Background worker

   # Production
   npm run build
   npm start &        # API server
   npm run worker &   # Background worker
   ```

## API Endpoints

### Health Check

`GET /health`

```bash
curl -X GET http://localhost:3000/health
```

Response:

```json
{
  "status": "healthy",
  "timestamp": "2025-06-29T12:00:00.000Z",
  "version": "1.0.0"
}
```

### Create Username

`POST /api/username`

```bash
curl -X POST http://localhost:3000/api/username \
  -H "Authorization: Bearer your_api_key" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "offer": "lno1pqq..."
  }'
```

Response:

```json
{
  "success": true,
  "data": {
    "requestId": "550e8400-e29b-41d4-a716-446655440000",
    "status": "pending",
    "bip353Address": "alice@yourdomain.com",
    "estimatedCompletionTime": "2025-06-29T12:30:00.000Z"
  }
}
```

### Check Status

`GET /api/status/:requestId`

```bash
curl -X GET http://localhost:3000/api/status/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer your_api_key"
```

Response:

```json
{
  "success": true,
  "data": {
    "requestId": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "bip353Address": "alice@yourdomain.com",
    "createdAt": "2025-06-29T12:00:00.000Z",
    "completedAt": "2025-06-29T12:00:30.000Z",
    "recordId": "abc123"
  }
}
```

## Scripts

### Development

- `npm run dev` - Start API server with hot reload
- `npm run dev:worker` - Start worker with hot reload

### Production

- `npm run build` - Compile TypeScript to JavaScript
- `npm start` - Run production API server
- `npm run worker` - Run production worker

### Maintenance

- `npm run clean` - Remove build artifacts and cache files
- `npm run format` - Format all code with Prettier
- `npm run format:check` - Check if code is properly formatted
- `npm run lint` - Run formatting check and TypeScript validation
- `npm run typecheck` - Validate TypeScript without emitting files

### Combined Commands

```bash
# Clean build for production
npm run clean && npm run build

# Format and build
npm run format && npm run build

# Full validation
npm run lint && npm run build
```

## Environment Variables

| Variable                | Description                      | Example                        | Required |
| ----------------------- | -------------------------------- | ------------------------------ | -------- |
| `PORT`                  | Server port                      | `3000`                         | Yes      |
| `NODE_ENV`              | Environment mode                 | `development` or `production`  | Yes      |
| `DOMAIN`                | Your domain for BIP353 addresses | `yourdomain.com`               | Yes      |
| `REDIS_URL`             | Redis connection string          | `redis://localhost:6379`       | Yes      |
| `REDIS_QUEUE_NAME`      | Queue name for jobs              | `bip353_username_queue`        | Yes      |
| `CLOUDFLARE_API_TOKEN`  | Cloudflare API token             | `your_token_here`              | Yes      |
| `CLOUDFLARE_ZONE_ID`    | Cloudflare zone ID               | `your_zone_id_here`            | Yes      |
| `API_KEYS`              | Comma-separated API keys         | `key1,key2,key3`               | Yes      |
| `WHITELISTED_IPS`       | Comma-separated allowed IPs      | `127.0.0.1,::1,192.168.1.0/24` | Yes      |
| `WORKER_MAX_RETRIES`    | Max retry attempts for jobs      | `3`                            | Yes      |
| `WORKER_RETRY_DELAY_MS` | Delay between retries (ms)       | `1000`                         | Yes      |

## Architecture

```
Client Request
     ↓
API Server (Express + TypeScript)
     ↓ (Authentication & Validation)
Redis Queue (Job Storage)
     ↓
Background Worker
     ↓ (Retry Logic)
Cloudflare API
     ↓
DNS TXT Record Created
```

## Job Processing

1. **Request Received** - API validates and queues job
2. **Job Processing** - Worker picks up job from Redis queue
3. **Cloudflare Integration** - Worker creates DNS TXT record
4. **Status Updates** - Real-time status tracking via Redis
5. **Error Handling** - Automatic retries with exponential backoff

## Status Flow

```
pending → processing → completed
   ↓           ↓
   ↓        failed (with retry)
   ↓           ↓
   ↓        failed (max retries exceeded)
   ↓
failed (validation error)
```

## Development

### Prerequisites

- Node.js >= 18.0.0
- Redis server
- Cloudflare account with API access

### Local Development Setup

```bash
# 1. Clone and install
git clone <repository>
cd merchant-usernames
npm install

# 2. Setup environment
cp .env.example .env
# Edit .env with your configuration

# 3. Start Redis (using Docker)
docker run -d --name redis -p 6379:6379 redis:7-alpine

# 4. Start development servers (2 terminals)
npm run dev        # Terminal 1: API server
npm run dev:worker # Terminal 2: Background worker
```

### Code Quality

```bash
# Format code
npm run format

# Check formatting
npm run format:check

# Type checking
npm run typecheck

# Full validation
npm run lint
```

## Production Deployment

1. **Build application**

   ```bash
   npm run clean
   npm run build
   ```

2. **Set environment variables** (production values)

3. **Start services**

   ```bash
   # Option 1: Direct execution
   npm start &        # API server
   npm run worker &   # Background worker

   # Option 2: Process manager
   pm2 start dist/server.js --name "bip353-api"
   pm2 start dist/worker.js --name "bip353-worker"
   ```

## Monitoring

- **Logs**: Check `logs/combined.log` and `logs/error.log`
- **Health Check**: `GET /health` endpoint
- **Redis**: Monitor queue size and processing times
- **Job Status**: Track via status endpoint

## Troubleshooting

### Common Issues

1. **Redis Connection Failed**
   - Verify Redis is running: `redis-cli ping`
   - Check REDIS_URL in environment

2. **Cloudflare API Errors**
   - Verify API token has correct permissions
   - Check zone ID is correct
   - Ensure domain is managed by Cloudflare

3. **Worker Not Processing Jobs**
   - Check worker logs for errors
   - Verify Redis connection
   - Restart worker process

### Debug Mode

Set `NODE_ENV=development` for detailed logging.

