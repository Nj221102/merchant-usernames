#!/bin/bash

# Simple API Test Script for Greenlight Backend
# Tests all main endpoints without WebSocket functionality

set -e  # Exit on any error

# Configuration
BASE_URL="http://localhost:8080"
TEST_USER="test_user_$(date +%s)"
TEST_PASSWORD="testpassword123"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Variables
USER_TOKEN=""
ENCRYPTED_SEED=""

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test functions
test_health() {
    log_info "Testing health endpoint..."
    response=$(curl -s "$BASE_URL/health" || echo "FAILED")
    
    if [ "$response" = "OK" ]; then
        log_success "Health check passed"
        return 0
    else
        log_error "Health check failed: $response"
        return 1
    fi
}

test_register() {
    log_info "Testing user registration..."
    response=$(curl -s -X POST "$BASE_URL/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"public_key\": \"$TEST_USER\", \"password\": \"$TEST_PASSWORD\"}")
    
    # Extract token and encrypted seed (note: server uses camelCase "encryptedSeed")
    USER_TOKEN=$(echo "$response" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    ENCRYPTED_SEED=$(echo "$response" | grep -o '"encryptedSeed":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$USER_TOKEN" ] && [ -n "$ENCRYPTED_SEED" ]; then
        log_success "User registration successful"
        return 0
    else
        log_error "User registration failed: $response"
        return 1
    fi
}

test_login() {
    log_info "Testing user login..."
    response=$(curl -s -X POST "$BASE_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"public_key\": \"$TEST_USER\", \"password\": \"$TEST_PASSWORD\"}")
    
    # Update token
    NEW_TOKEN=$(echo "$response" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$NEW_TOKEN" ]; then
        USER_TOKEN="$NEW_TOKEN"
        log_success "User login successful"
        return 0
    else
        log_error "User login failed: $response"
        return 1
    fi
}

test_node_register() {
    log_info "Testing node registration..."
    response=$(curl -s -X POST "$BASE_URL/node/register" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $USER_TOKEN" \
        -d "{\"encryptedSeed\": \"$ENCRYPTED_SEED\", \"password\": \"$TEST_PASSWORD\"}")
    
    if echo "$response" | grep -q "encryptedDeviceCreds\|node_id\|nodeId\|success"; then
        log_success "Node registration successful"
        return 0
    else
        log_error "Node registration failed: $response"
        return 1
    fi
}

test_node_info() {
    log_info "Testing node info..."
    response=$(curl -s -X GET "$BASE_URL/node/info" \
        -H "Authorization: Bearer $USER_TOKEN")
    
    if echo "$response" | grep -q "node_id\|nodeId\|id"; then
        log_success "Node info retrieved successfully"
        return 0
    else
        log_error "Node info failed: $response"
        return 1
    fi
}

test_node_balance() {
    log_info "Testing node balance..."
    response=$(curl -s -X GET "$BASE_URL/node/balance" \
        -H "Authorization: Bearer $USER_TOKEN")
    
    if echo "$response" | grep -q "balance\|total\|msat"; then
        log_success "Node balance retrieved successfully"
        return 0
    else
        log_error "Node balance failed: $response"
        return 1
    fi
}

test_create_offer() {
    log_info "Testing Bolt12 offer creation..."
    response=$(curl -s -X POST "$BASE_URL/node/offer" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $USER_TOKEN" \
        -d '{"amount": 1000, "description": "Test offer"}')
    
    if echo "$response" | grep -q "offer\|lno1"; then
        log_success "Bolt12 offer created successfully"
        return 0
    else
        log_error "Bolt12 offer creation failed: $response"
        return 1
    fi
}

# Main execution
main() {
    echo "=========================================="
    echo "  Greenlight Backend API Test Suite"
    echo "=========================================="
    echo "Testing against: $BASE_URL"
    echo "Test user: $TEST_USER"
    echo ""
    
    # Track test results
    passed=0
    failed=0
    
    # Run tests
    for test in test_health test_register test_login test_node_register test_node_info test_node_balance test_create_offer; do
        if $test; then
            ((passed++))
        else
            ((failed++))
        fi
        echo ""
    done
    
    # Summary
    echo "=========================================="
    echo "Test Results:"
    echo "  Passed: $passed"
    echo "  Failed: $failed"
    echo "  Total:  $((passed + failed))"
    echo "=========================================="
    
    if [ $failed -eq 0 ]; then
        log_success "All tests passed!"
        exit 0
    else
        log_error "Some tests failed!"
        exit 1
    fi
}

# Check dependencies
check_deps() {
    for cmd in curl grep cut; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command '$cmd' not found"
            exit 1
        fi
    done
}

# Help function
show_help() {
    echo "Greenlight Backend API Test Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -u, --url URL  Set base URL (default: http://localhost:8080)"
    echo ""
    echo "This script tests all main API endpoints:"
    echo "  - Health check"
    echo "  - User registration"
    echo "  - User login"
    echo "  - Node registration"
    echo "  - Node information"
    echo "  - Node balance"
    echo "  - Bolt12 offer creation"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Run the tests
check_deps
main
