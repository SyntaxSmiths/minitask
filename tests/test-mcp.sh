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
echo "=== Running MCP test program ==="
MINITASK_GUI_TASK_FILE="$TEST_FILE" ./target/debug/minitask --gui --file "$TEST_FILE" &
MINITASK_PID=$!

# Give it a moment to start
sleep 1

# Check if it's still running
if ! kill -0 $MINITASK_PID 2>/dev/null; then
    echo "ERROR: minitask GUI process died"
    exit 1
fi

echo "Minitask GUI running with PID $MINITASK_PID"
echo "Press Ctrl+C to stop"

# Wait for user to stop
wait $MINITASK_PID
