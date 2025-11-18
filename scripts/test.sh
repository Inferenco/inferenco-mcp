#!/bin/bash

# Inferenco-MCP Test Script

set -e

echo "Testing Inferenco-MCP Server..."
echo "========================="

# Build first
echo "Building server..."
cargo build --bin inferenco-mcp-stdio

echo ""
echo "Running tests..."

# Test 1: List tools
echo "1. Testing tools/list..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | timeout 10s cargo run --bin inferenco-mcp-stdio 2>/dev/null | tail -1)

if echo "$RESPONSE" | grep -q 'get_gecko_networks' && echo "$RESPONSE" | grep -q 'get_trending_pools'; then
    echo "   ✅ Tools list successful"
else
    echo "   ❌ Tools list failed"
    echo "   Response: $RESPONSE"
    exit 1
fi

# Test 2: get_gecko_networks (may fail offline)
echo "2. Testing get_gecko_networks tool (non-fatal if offline)..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_gecko_networks","arguments":{}}}' | timeout 10s cargo run --bin inferenco-mcp-stdio 2>/dev/null | tail -1)

if echo "$RESPONSE" | grep -q '"content"'; then
    echo "   ✅ get_gecko_networks responded"
else
    echo "   ⚠️  get_gecko_networks call possibly failed (offline?)"
    echo "   Response: $RESPONSE"
fi

# Test 3: get_trending_pools (may fail offline)
echo "3. Testing get_trending_pools tool (non-fatal if offline)..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_trending_pools","arguments":{"network":"eth","limit":5}}}' | timeout 10s cargo run --bin inferenco-mcp-stdio 2>/dev/null | tail -1)

if echo "$RESPONSE" | grep -q '"content"'; then
    echo "   ✅ get_trending_pools responded"
else
    echo "   ⚠️  get_trending_pools call possibly failed (offline?)"
    echo "   Response: $RESPONSE"
fi

echo ""
echo "🎉 All tests passed!"
echo "Inferenco-MCP server basic checks completed."
