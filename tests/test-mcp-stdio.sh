#!/bin/bash
set -e

echo "=== Building minitask ==="
cargo build

echo ""
echo "=== Compiling TypeScript test ==="
cd ui
npx tsc mcp-test.ts --target ES2020 --module ES2020 --lib ES2020 --skipLibCheck
cd ..

echo ""
echo "=== Creating test task file ==="
TEST_FILE="/tmp/minitask-test.toml"
rm -f "$TEST_FILE"
./target/debug/minitask --file "$TEST_FILE" new --content "Initial test task"
./target/debug/minitask --file "$TEST_FILE" new --content "Second test task"

echo ""
echo "=== Current tasks in file ==="
./target/debug/minitask --file "$TEST_FILE" list --verbose

echo ""
echo "=== Testing MCP communication ==="
echo "Sending initialize request..."

# Test basic MCP communication
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/debug/minitask --mcp

echo ""
echo "=== Testing list tool ==="
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
  echo "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"list\",\"arguments\":{\"file\":\"$TEST_FILE\",\"verbose\":true}}}"
) | ./target/debug/minitask --mcp
