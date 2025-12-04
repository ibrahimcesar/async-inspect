#!/usr/bin/env bash
# Verify package is ready for crates.io publication
# Run this before `cargo publish`

set -e
set -u

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}   async-inspect Package Verification${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

FAILED=0

# Check 1: Git status
echo -e "${YELLOW}[1/10] Checking git status...${NC}"
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}✗ Working directory is not clean${NC}"
    echo "  Uncommitted changes found. Commit them first."
    git status --short
    FAILED=1
else
    echo -e "${GREEN}✓ Working directory is clean${NC}"
fi
echo ""

# Check 2: Version consistency
echo -e "${YELLOW}[2/10] Checking version consistency...${NC}"
MAIN_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
MACRO_VERSION=$(grep '^version = ' async-inspect-macros/Cargo.toml | head -1 | cut -d'"' -f2)
MACRO_DEP_VERSION=$(grep 'async-inspect-macros = { version' Cargo.toml | sed 's/.*version = "\([^"]*\)".*/\1/')

echo "  Main crate version: $MAIN_VERSION"
echo "  Macro crate version: $MACRO_VERSION"
echo "  Macro dependency version: $MACRO_DEP_VERSION"

if [ "$MAIN_VERSION" != "$MACRO_VERSION" ]; then
    echo -e "${RED}✗ Version mismatch between main and macro crate${NC}"
    FAILED=1
elif [ "$MACRO_VERSION" != "$MACRO_DEP_VERSION" ]; then
    echo -e "${RED}✗ Macro dependency version doesn't match${NC}"
    FAILED=1
else
    echo -e "${GREEN}✓ All versions consistent: $MAIN_VERSION${NC}"
fi
echo ""

# Check 3: Cargo metadata
echo -e "${YELLOW}[3/10] Checking cargo metadata...${NC}"
METADATA_OK=1

if ! grep -q '^description = ' Cargo.toml; then
    echo -e "${RED}✗ Missing description in Cargo.toml${NC}"
    METADATA_OK=0
fi

if ! grep -q '^license = ' Cargo.toml; then
    echo -e "${RED}✗ Missing license in Cargo.toml${NC}"
    METADATA_OK=0
fi

if ! grep -q '^repository = ' Cargo.toml; then
    echo -e "${RED}✗ Missing repository in Cargo.toml${NC}"
    METADATA_OK=0
fi

if [ $METADATA_OK -eq 1 ]; then
    echo -e "${GREEN}✓ Cargo metadata complete${NC}"
else
    FAILED=1
fi
echo ""

# Check 4: Required files
echo -e "${YELLOW}[4/10] Checking required files...${NC}"
FILES_OK=1

if [ ! -f "README.md" ]; then
    echo -e "${RED}✗ Missing README.md${NC}"
    FILES_OK=0
else
    echo -e "${GREEN}✓ README.md exists${NC}"
fi

if [ ! -f "LICENSE" ]; then
    echo -e "${RED}✗ Missing LICENSE${NC}"
    FILES_OK=0
else
    echo -e "${GREEN}✓ LICENSE exists${NC}"
fi

if [ ! -f "CHANGELOG.md" ]; then
    echo -e "${YELLOW}⚠ Missing CHANGELOG.md (recommended)${NC}"
else
    echo -e "${GREEN}✓ CHANGELOG.md exists${NC}"
fi

if [ $FILES_OK -eq 0 ]; then
    FAILED=1
fi
echo ""

# Check 5: Tests
echo -e "${YELLOW}[5/10] Running tests...${NC}"
if cargo test --all-features --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ All tests passed${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    FAILED=1
fi
echo ""

# Check 6: Clippy
echo -e "${YELLOW}[6/10] Running clippy...${NC}"
if cargo clippy --all-features -- -D warnings > /dev/null 2>&1; then
    echo -e "${GREEN}✓ No clippy warnings${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    FAILED=1
fi
echo ""

# Check 7: Formatting
echo -e "${YELLOW}[7/10] Checking code formatting...${NC}"
if cargo fmt --all -- --check > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Code is formatted${NC}"
else
    echo -e "${RED}✗ Code needs formatting${NC}"
    echo "  Run: cargo fmt --all"
    FAILED=1
fi
echo ""

# Check 8: Documentation
echo -e "${YELLOW}[8/10] Building documentation...${NC}"
if RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Documentation builds without warnings${NC}"
else
    echo -e "${RED}✗ Documentation build has warnings${NC}"
    FAILED=1
fi
echo ""

# Check 9: Package contents (proc macro)
echo -e "${YELLOW}[9/10] Verifying proc macro package...${NC}"
cd async-inspect-macros
if cargo package --list --allow-dirty > /dev/null 2>&1; then
    FILE_COUNT=$(cargo package --list --allow-dirty 2>/dev/null | wc -l)
    echo -e "${GREEN}✓ Proc macro package OK ($FILE_COUNT files)${NC}"
else
    echo -e "${RED}✗ Proc macro package verification failed${NC}"
    FAILED=1
fi
cd ..
echo ""

# Check 10: Package contents (main crate)
echo -e "${YELLOW}[10/10] Verifying main package...${NC}"
if cargo package --list --allow-dirty > /dev/null 2>&1; then
    FILE_COUNT=$(cargo package --list --allow-dirty 2>/dev/null | wc -l)
    echo -e "${GREEN}✓ Main package OK ($FILE_COUNT files)${NC}"

    # Show what will be published
    echo ""
    echo -e "${BLUE}Package will include:${NC}"
    cargo package --list --allow-dirty 2>/dev/null | head -20
    TOTAL=$(cargo package --list --allow-dirty 2>/dev/null | wc -l)
    if [ $TOTAL -gt 20 ]; then
        echo "  ... and $((TOTAL - 20)) more files"
    fi
else
    echo -e "${RED}✗ Main package verification failed${NC}"
    FAILED=1
fi
echo ""

# Final result
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! Package is ready for publication.${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo ""
    echo "  1. Commit any remaining changes:"
    echo "     ${BLUE}git add -A && git commit -m 'chore: prepare for v${MAIN_VERSION} release'${NC}"
    echo ""
    echo "  2. Create and push git tag:"
    echo "     ${BLUE}git tag v${MAIN_VERSION}${NC}"
    echo "     ${BLUE}git push origin main --tags${NC}"
    echo ""
    echo "  3. Publish proc macro crate first:"
    echo "     ${BLUE}cd async-inspect-macros && cargo publish${NC}"
    echo ""
    echo "  4. Wait 2 minutes, then publish main crate:"
    echo "     ${BLUE}cd .. && cargo publish${NC}"
    echo ""
    echo "  5. Create GitHub release:"
    echo "     ${BLUE}gh release create v${MAIN_VERSION} --title 'v${MAIN_VERSION}' --notes-file CHANGELOG.md${NC}"
    echo ""
else
    echo -e "${RED}✗ Some checks failed. Fix the issues above before publishing.${NC}"
    exit 1
fi
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
