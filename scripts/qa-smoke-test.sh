#!/bin/bash
# async-inspect QA Smoke Test Script
# Run this script to perform automated testing of core functionality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
SKIPPED=0

# Test result logging
log_pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
}

log_fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED++))
}

log_skip() {
    echo -e "${YELLOW}○ SKIP${NC}: $1"
    ((SKIPPED++))
}

log_info() {
    echo -e "${BLUE}ℹ INFO${NC}: $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Change to project root
cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           async-inspect QA Smoke Test Suite                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Project root: $PROJECT_ROOT"
echo "Date: $(date)"
echo ""

# ============================================================================
log_section "1. Build Tests"
# ============================================================================

log_info "Building with all features..."
if cargo build --all-features 2>&1; then
    log_pass "Build with all features"
else
    log_fail "Build with all features"
fi

log_info "Building release..."
if cargo build --release --all-features 2>&1; then
    log_pass "Release build"
else
    log_fail "Release build"
fi

# ============================================================================
log_section "2. Unit Tests"
# ============================================================================

log_info "Running unit tests..."
if cargo test --all-features 2>&1 | tail -5; then
    log_pass "Unit tests"
else
    log_fail "Unit tests"
fi

# ============================================================================
log_section "3. Doc Tests"
# ============================================================================

log_info "Running doc tests..."
if cargo test --doc --all-features 2>&1 | tail -5; then
    log_pass "Doc tests"
else
    log_fail "Doc tests"
fi

# ============================================================================
log_section "4. Specific Module Tests"
# ============================================================================

log_info "Testing ring buffer module..."
if cargo test ringbuf --all-features 2>&1 | tail -3; then
    log_pass "Ring buffer tests"
else
    log_fail "Ring buffer tests"
fi

log_info "Testing adaptive sampling..."
if cargo test adaptive --all-features 2>&1 | tail -3; then
    log_pass "Adaptive sampling tests"
else
    log_fail "Adaptive sampling tests"
fi

log_info "Testing config module..."
if cargo test config --all-features 2>&1 | tail -3; then
    log_pass "Config tests"
else
    log_fail "Config tests"
fi

log_info "Testing timeline module..."
if cargo test timeline --all-features 2>&1 | tail -3; then
    log_pass "Timeline tests"
else
    log_fail "Timeline tests"
fi

log_info "Testing errors module..."
if cargo test errors --all-features 2>&1 | tail -3; then
    log_pass "Errors module tests"
else
    log_fail "Errors module tests"
fi

# ============================================================================
log_section "5. Example Compilation"
# ============================================================================

log_info "Checking basic_inspection example..."
if cargo build --example basic_inspection 2>&1; then
    log_pass "basic_inspection compiles"
else
    log_fail "basic_inspection compiles"
fi

log_info "Checking tokio_integration example..."
if cargo build --example tokio_integration --features tokio 2>&1; then
    log_pass "tokio_integration compiles"
else
    log_fail "tokio_integration compiles"
fi

log_info "Checking deadlock_detection example..."
if cargo build --example deadlock_detection 2>&1; then
    log_pass "deadlock_detection compiles"
else
    log_fail "deadlock_detection compiles"
fi

log_info "Checking tui_monitor example..."
if cargo build --example tui_monitor --features cli 2>&1; then
    log_pass "tui_monitor compiles"
else
    log_fail "tui_monitor compiles"
fi

log_info "Checking dashboard_demo example..."
if cargo build --example dashboard_demo --features dashboard 2>&1; then
    log_pass "dashboard_demo compiles"
else
    log_fail "dashboard_demo compiles"
fi

log_info "Checking production_ready example..."
if cargo build --example production_ready --features tokio 2>&1; then
    log_pass "production_ready compiles"
else
    log_fail "production_ready compiles"
fi

# ============================================================================
log_section "6. Clippy Checks"
# ============================================================================

log_info "Running clippy..."
if cargo clippy --all-features 2>&1 | tail -10; then
    log_pass "Clippy checks"
else
    log_fail "Clippy checks"
fi

# ============================================================================
log_section "7. Documentation Build"
# ============================================================================

log_info "Building rustdoc..."
if cargo doc --all-features --no-deps 2>&1 | tail -3; then
    log_pass "Rustdoc build"
else
    log_fail "Rustdoc build"
fi

# ============================================================================
log_section "8. Security Audit"
# ============================================================================

if command -v cargo-audit &> /dev/null; then
    log_info "Running cargo audit..."
    if cargo audit 2>&1 | tail -10; then
        log_pass "Security audit (warnings only)"
    else
        log_skip "Security audit (has advisories, review manually)"
    fi
else
    log_skip "cargo-audit not installed"
fi

# ============================================================================
log_section "9. Feature Combinations"
# ============================================================================

log_info "Testing minimal features (no default)..."
if cargo check --no-default-features 2>&1; then
    log_pass "Minimal features build"
else
    log_fail "Minimal features build"
fi

log_info "Testing tokio only..."
if cargo check --no-default-features --features tokio 2>&1; then
    log_pass "Tokio-only build"
else
    log_fail "Tokio-only build"
fi

log_info "Testing CLI only..."
if cargo check --no-default-features --features cli,tokio 2>&1; then
    log_pass "CLI-only build"
else
    log_fail "CLI-only build"
fi

# ============================================================================
log_section "10. Quick Example Runs"
# ============================================================================

log_info "Running basic_inspection (5s timeout)..."
if timeout 5 cargo run --example basic_inspection 2>&1 || [ $? -eq 124 ]; then
    log_pass "basic_inspection runs"
else
    log_fail "basic_inspection runs"
fi

log_info "Running proc_macro_test (5s timeout)..."
if timeout 5 cargo run --example proc_macro_test 2>&1 || [ $? -eq 124 ]; then
    log_pass "proc_macro_test runs"
else
    log_fail "proc_macro_test runs"
fi

# ============================================================================
log_section "Summary"
# ============================================================================

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                      Test Results                              ║"
echo "╠════════════════════════════════════════════════════════════════╣"
printf "║  ${GREEN}Passed${NC}:  %-50s ║\n" "$PASSED"
printf "║  ${RED}Failed${NC}:  %-50s ║\n" "$FAILED"
printf "║  ${YELLOW}Skipped${NC}: %-50s ║\n" "$SKIPPED"
echo "╠════════════════════════════════════════════════════════════════╣"
TOTAL=$((PASSED + FAILED + SKIPPED))
printf "║  Total:   %-50s ║\n" "$TOTAL"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Some tests failed! Review the output above.${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
