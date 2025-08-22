# Setup Guide

Complete setup guide for the Greenlight Lightning Wallet Backend.

## Prerequisites

### Required Software

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **PostgreSQL** (version 12 or higher)
   ```bash
   # macOS
   brew install postgresql
   brew services start postgresql
   
   # Ubuntu/Debian
   sudo apt update
   sudo apt install postgresql postgresql-contrib
   sudo systemctl start postgresql
   ```

3. **Git**
   ```bash
   # macOS
   brew install git
   
   # Ubuntu/Debian
   sudo apt install git
   ```

### Required Tools

- `curl` - for testing API endpoints
- `openssl` - for cryptographic operations

## Project Setup

### 1. Clone Repository

```bash
git clone <repository-url>
cd greenlight-backend
```

### 2. Database Setup

Create a PostgreSQL database:

```bash
# Connect to PostgreSQL
psql postgres

# Create database and user
CREATE DATABASE greenlight_wallet;
CREATE USER greenlight_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE greenlight_wallet TO greenlight_user;
\q
```

### 3. Environment Configuration

Create a `.env` file from the example:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/greenlight_wallet

# JWT Configuration (generate a secure secret)
JWT_SECRET=your-super-secret-jwt-key-change-in-production-must-be-at-least-32-chars

# Server Configuration
HOST=0.0.0.0
PORT=8080

# Greenlight Credentials
GL_CERT_PATH=./client.crt
GL_KEY_PATH=./client-key.pem
GL_NETWORK=bitcoin
```

### 4. Greenlight Credentials

You need Greenlight developer credentials from Blockstream:

1. **Get Greenlight credentials** from Blockstream
2. **Place credential files** in the project root:
   - `client.crt` - Your developer certificate
   - `client-key.pem` - Your developer private key

**Alternative**: Use environment variables:
```bash
# Convert files to base64 and set environment variables
export GL_CERT_CONTENT=$(base64 -i client.crt)
export GL_KEY_CONTENT=$(base64 -i client-key.pem)
```

### 5. Install Dependencies

```bash
cargo build
```

### 6. Run Database Migrations

Database migrations run automatically when the server starts:

```bash
cargo run
```

## Running the Server

### Development Mode

```bash
cargo run
```

### Production Mode

```bash
cargo run --release
```

The server will start on `http://localhost:8080` by default.

## Testing

### Automated Testing

Use the provided test script to test all endpoints:

```bash
# Make script executable
chmod +x test_endpoints.sh

# Run tests
./test_endpoints.sh
```

The script will:
1. Check server health
2. Register a test user
3. Login with test credentials
4. Register a Lightning node
5. Get node information
6. Check node balance
7. Create a Bolt12 offer

### Manual Testing

#### 1. Check Server Health

```bash
curl http://localhost:8080/health
```

Expected response: `OK`

#### 2. Register User

```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "public_key": "test_user_12345",
    "password": "testpassword123"
  }'
```

Expected response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "encrypted_seed": "encrypted_seed_data_here"
}
```

#### 3. Login User

```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "public_key": "test_user_12345",
    "password": "testpassword123"
  }'
```

#### 4. Register Lightning Node

```bash
# Save the token from login response
TOKEN="your_jwt_token_here"

curl -X POST http://localhost:8080/node/register \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "encrypted_seed": "your_encrypted_seed_here",
    "password": "testpassword123"
  }'
```

#### 5. Get Node Info

```bash
curl -X GET http://localhost:8080/node/info \
  -H "Authorization: Bearer $TOKEN"
```

#### 6. Check Node Balance

```bash
curl -X GET http://localhost:8080/node/balance \
  -H "Authorization: Bearer $TOKEN"
```

#### 7. Create Bolt12 Offer

```bash
curl -X POST http://localhost:8080/node/offer \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "amount": 1000,
    "description": "Test Bolt12 offer"
  }'
```

## Troubleshooting

### Common Issues

#### Database Connection Error

```
Error: Failed to connect to database
```

**Solution**:
- Check PostgreSQL is running: `brew services list | grep postgresql`
- Verify database connection details in the application
- Ensure database and user exist

#### Greenlight Authentication Error

```
Error: Failed to authenticate with Greenlight
```

**Solution**:
- Verify credential files exist: `ls -la client.*`
- Check file permissions: `chmod 600 client.crt client-key.pem`
- Ensure credentials are valid Greenlight developer credentials

#### Port Already in Use

```
Error: Address already in use
```

**Solution**:
- Kill existing process: `lsof -ti:8080 | xargs kill`
- Or modify the port in the application code

#### JWT Token Error

```
Error: Invalid token
```

**Solution**:
- Use a new token from fresh login
- Ensure token is passed in Authorization header correctly

### Logs

Enable debug logging:

```bash
RUST_LOG=debug cargo run
```

View specific module logs:

```bash
RUST_LOG=greenlight_backend=debug cargo run
```

### Database Issues

Reset database:

```bash
# Drop and recreate database
psql postgres -c "DROP DATABASE IF EXISTS greenlight_wallet;"
psql postgres -c "CREATE DATABASE greenlight_wallet;"
psql postgres -c "GRANT ALL PRIVILEGES ON DATABASE greenlight_wallet TO greenlight_user;"

# Run migrations again (they run automatically on server start)
cargo run
```

## Available Endpoints

The backend provides the following API endpoints:

- `GET /health` - Health check
- `POST /auth/register` - Register new user
- `POST /auth/login` - User login
- `POST /node/register` - Register Lightning node
- `POST /node/recover` - Recover Lightning node
- `GET /node/info` - Get node information
- `GET /node/balance` - Get node balance
- `POST /node/offer` - Create Bolt12 offer
- `GET /ws` - WebSocket connection (for real-time updates)

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── error.rs             # Error handling
├── models/              # Database models
├── handlers/            # HTTP request handlers
├── services/            # Business logic
└── middleware/          # Custom middleware

migrations/              # Database migrations
Cargo.toml              # Rust dependencies
```

### Running Tests

```bash
# Build and run tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for errors
cargo check
```