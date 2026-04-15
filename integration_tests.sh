#!/bin/bash

# Integration tests for minitask CLI
# Tests all commands end-to-end

set -e

TEST_FILE="test_integration.toml"
MINITASK="./target/debug/minitask"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Test counter
TESTS_RUN=0
TESTS_PASSED=0

# Cleanup function
cleanup() {
    rm -f "$TEST_FILE"
}

# Setup
setup() {
    cleanup
    cargo build --quiet 2>/dev/null || cargo build
}

# Test helper
assert_equals() {
    local expected="$1"
    local actual="$2"
    local test_name="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ "$expected" = "$actual" ]; then
        echo -e "${GREEN}✓${NC} $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗${NC} $test_name"
        echo "  Expected: $expected"
        echo "  Actual: $actual"
    fi
}

assert_contains() {
    local substring="$1"
    local text="$2"
    local test_name="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if echo "$text" | grep -q "$substring"; then
        echo -e "${GREEN}✓${NC} $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗${NC} $test_name"
        echo "  Expected substring: $substring"
        echo "  In text: $text"
    fi
}

# Run tests
echo "Running minitask integration tests..."
echo

setup

# Test 1: Create new task
echo "Test: new command"
output=$($MINITASK --file "$TEST_FILE" new "First task")
assert_contains "Created TASK-0" "$output" "Create first task"

# Test 2: Create second task
output=$($MINITASK --file "$TEST_FILE" new "Second task")
assert_contains "Created TASK-1" "$output" "Create second task"

# Test 3: List all tasks
output=$($MINITASK --file "$TEST_FILE" list)
assert_contains "TASK-0" "$output" "List shows TASK-0"
assert_contains "TASK-1" "$output" "List shows TASK-1"

# Test 4: Show specific task
output=$($MINITASK --file "$TEST_FILE" show TASK-0)
assert_contains "TASK-0" "$output" "Show TASK-0"
assert_contains "First task" "$output" "Show contains content"

# Test 5: Edit state
$MINITASK --file "$TEST_FILE" edit state TASK-0 in-progress
output=$($MINITASK --file "$TEST_FILE" show TASK-0 --verbose)
assert_contains "in-progress" "$output" "State changed to in-progress"

# Test 6: Edit content
$MINITASK --file "$TEST_FILE" edit content TASK-0 "Updated content"
output=$($MINITASK --file "$TEST_FILE" show TASK-0)
assert_contains "Updated content" "$output" "Content updated"

# Test 7: Add content
$MINITASK --file "$TEST_FILE" add content TASK-0 "\nAppended text"
output=$($MINITASK --file "$TEST_FILE" show TASK-0)
assert_contains "Appended text" "$output" "Content appended"

# Test 8: Add depends-on
$MINITASK --file "$TEST_FILE" add depends-on TASK-0 TASK-1
output=$($MINITASK --file "$TEST_FILE" show TASK-0 --verbose)
assert_contains "TASK-1" "$output" "Dependency added"

# Test 9: Del depends-on
$MINITASK --file "$TEST_FILE" del depends-on TASK-0 TASK-1
output=$($MINITASK --file "$TEST_FILE" show TASK-0 --verbose)
if echo "$output" | grep -q "Depends on:.*TASK-1"; then
    echo -e "${RED}✗${NC} Dependency removed"
    TESTS_RUN=$((TESTS_RUN + 1))
else
    echo -e "${GREEN}✓${NC} Dependency removed"
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi

# Test 10: Add epic
$MINITASK --file "$TEST_FILE" add epic TASK-0 planning
output=$($MINITASK --file "$TEST_FILE" show TASK-0 --verbose)
assert_contains "planning" "$output" "Epic added"

# Test 11: Del epic
$MINITASK --file "$TEST_FILE" del epic TASK-0 planning
output=$($MINITASK --file "$TEST_FILE" show TASK-0 --verbose)
if echo "$output" | grep -q "Epic:.*planning"; then
    echo -e "${RED}✗${NC} Epic removed"
    TESTS_RUN=$((TESTS_RUN + 1))
else
    echo -e "${GREEN}✓${NC} Epic removed"
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi

# Test 12: List with state filter
$MINITASK --file "$TEST_FILE" edit state TASK-1 done
output=$($MINITASK --file "$TEST_FILE" list --state done)
assert_contains "TASK-1" "$output" "List filtered by state"
if echo "$output" | grep -q "TASK-0"; then
    echo -e "${RED}✗${NC} List state filter excludes other states"
    TESTS_RUN=$((TESTS_RUN + 1))
else
    echo -e "${GREEN}✓${NC} List state filter excludes other states"
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi

# Test 13: List with epic filter
$MINITASK --file "$TEST_FILE" add epic TASK-1 testing
output=$($MINITASK --file "$TEST_FILE" list --epic testing)
assert_contains "TASK-1" "$output" "List filtered by epic"

# Test 14: Claim command
$MINITASK --file "$TEST_FILE" new "Third task"
output=$($MINITASK --file "$TEST_FILE" claim in-progress)
assert_contains "TASK-2" "$output" "Claim task from todo"
assert_contains "in-progress" "$output" "Claimed task moved to in-progress"

# Test 15: Claim with dependency blocking
$MINITASK --file "$TEST_FILE" new "Fourth task"
$MINITASK --file "$TEST_FILE" new "Fifth task"
$MINITASK --file "$TEST_FILE" add depends-on TASK-3 TASK-4
$MINITASK --file "$TEST_FILE" edit state TASK-2 done
output=$($MINITASK --file "$TEST_FILE" claim in-progress)
assert_contains "TASK-4" "$output" "Claim skips blocked task"

# Test 16: JSON output
output=$($MINITASK --file "$TEST_FILE" show TASK-0 --json-out)
assert_contains '"name"' "$output" "JSON output contains name field"
assert_contains '"state"' "$output" "JSON output contains state field"

# Test 17: Stdin input for new command
echo "Task from stdin" | $MINITASK --file "$TEST_FILE" new -
output=$($MINITASK --file "$TEST_FILE" list)
assert_contains "Task from stdin" "$output" "Task created from stdin"

# Test 18: Verbose list
output=$($MINITASK --file "$TEST_FILE" list --verbose)
assert_contains "State:" "$output" "Verbose list shows state"
assert_contains "Content:" "$output" "Verbose list shows content"

# Cleanup
cleanup

# Summary
echo
echo "================================"
echo "Tests run: $TESTS_RUN"
echo "Tests passed: $TESTS_PASSED"
echo "Tests failed: $((TESTS_RUN - TESTS_PASSED))"
echo "================================"

if [ $TESTS_RUN -eq $TESTS_PASSED ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
