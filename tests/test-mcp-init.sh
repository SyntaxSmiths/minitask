#!/bin/bash
# Test script to inspect MCP initialization response

set -e

TEST_FILE="test-tasks-init.toml"
rm -f "$TEST_FILE"

echo "Starting minitask MCP server and capturing initialization response..."

# Start MCP server in background
./target/release/minitask --mcp --file "$TEST_FILE" 2>/dev/null &
MCP_PID=$!

# Give it time to start
sleep 0.5

# Send initialize request and capture response
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | \
  nc -U /proc/$MCP_PID/fd/0 2>/dev/null | \
  jq '.' || echo "Failed to get response"

# Cleanup
kill $MCP_PID 2>/dev/null || true
rm -f "$TEST_FILE"

echo ""
echo "Check if serverInfo contains version, name, etc."
