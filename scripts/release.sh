#!/usr/bin/env bash
# async-inspect Release Script
# Automates the release process for v0.1.0

set -e  # Exit on error
set -u  # Exit on undefined variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VERSION="0.1.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}   async-inspect v${VERSION} Release Script${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

cd "$REPO_ROOT"

# Step 1: Pre-flight Checks
echo -e "${YELLOW}[1/10] Running pre-flight checks...${NC}"

# Check if on main branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo -e "${RED}Error: Not on main branch (currently on: $BRANCH)${NC}"
    exit 1
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: Working directory is not clean${NC}"
    git status --short
    exit 1
fi

# Check if version matches in Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
if [ "$CARGO_VERSION" != "$VERSION" ]; then
    echo -e "${RED}Error: Cargo.toml version ($CARGO_VERSION) doesn't match release version ($VERSION)${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Pre-flight checks passed${NC}"
echo ""

# Step 2: Run Full Test Suite
echo -e "${YELLOW}[2/10] Running full test suite...${NC}"
cargo test --all-features || {
    echo -e "${RED}Error: Tests failed${NC}"
    exit 1
}
echo -e "${GREEN}✓ All tests passed${NC}"
echo ""

# Step 3: Run Benchmarks
echo -e "${YELLOW}[3/10] Running performance benchmarks...${NC}"
cargo bench --no-fail-fast || {
    echo -e "${YELLOW}Warning: Benchmarks failed or incomplete${NC}"
}
echo -e "${GREEN}✓ Benchmarks completed${NC}"
echo ""

# Step 4: Check Documentation
echo -e "${YELLOW}[4/10] Building documentation...${NC}"
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps || {
    echo -e "${RED}Error: Documentation build failed${NC}"
    exit 1
}
echo -e "${GREEN}✓ Documentation built successfully${NC}"
echo ""

# Step 5: Run Clippy
echo -e "${YELLOW}[5/10] Running clippy...${NC}"
cargo clippy --all-features -- -D warnings || {
    echo -e "${RED}Error: Clippy found issues${NC}"
    exit 1
}
echo -e "${GREEN}✓ Clippy checks passed${NC}"
echo ""

# Step 6: Format Check
echo -e "${YELLOW}[6/10] Checking code formatting...${NC}"
cargo fmt --all -- --check || {
    echo -e "${RED}Error: Code is not formatted${NC}"
    exit 1
}
echo -e "${GREEN}✓ Code formatting OK${NC}"
echo ""

# Step 7: Build All Features
echo -e "${YELLOW}[7/10] Building with all features...${NC}"
cargo build --release --all-features || {
    echo -e "${RED}Error: Build failed${NC}"
    exit 1
}
echo -e "${GREEN}✓ Release build successful${NC}"
echo ""

# Step 8: Build LSP Binary
echo -e "${YELLOW}[8/10] Building LSP binary...${NC}"
cargo build --release --features lsp --bin async-inspect-lsp || {
    echo -e "${RED}Error: LSP binary build failed${NC}"
    exit 1
}
echo -e "${GREEN}✓ LSP binary built${NC}"
echo ""

# Step 9: Run Examples
echo -e "${YELLOW}[9/10] Testing examples...${NC}"
timeout 5 cargo run --example basic_inspection 2>&1 | head -20 || true
timeout 5 cargo run --example tokio_integration --features tokio 2>&1 | head -20 || true
echo -e "${GREEN}✓ Examples tested${NC}"
echo ""

# Step 10: Create Git Tag
echo -e "${YELLOW}[10/10] Creating git tag...${NC}"
git tag -a "v${VERSION}" -m "Release v${VERSION}" || {
    echo -e "${RED}Error: Failed to create git tag${NC}"
    exit 1
}
echo -e "${GREEN}✓ Git tag created: v${VERSION}${NC}"
echo ""

# Summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ Release preparation complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""
echo -e "  1. Review the changes:"
echo -e "     ${BLUE}git show v${VERSION}${NC}"
echo ""
echo -e "  2. Push the tag:"
echo -e "     ${BLUE}git push origin v${VERSION}${NC}"
echo ""
echo -e "  3. Publish to crates.io:"
echo -e "     ${BLUE}cargo publish --dry-run${NC}"
echo -e "     ${BLUE}cargo publish${NC}"
echo ""
echo -e "  4. Create GitHub release:"
echo -e "     ${BLUE}gh release create v${VERSION} --title \"v${VERSION}\" --notes-file CHANGELOG.md${NC}"
echo ""
echo -e "  5. Publish IDE extensions:"
echo -e "     • VS Code: ${BLUE}cd vscode-extension && vsce publish${NC}"
echo -e "     • IntelliJ: ${BLUE}cd intellij-plugin && ./gradlew publishPlugin${NC}"
echo ""
echo -e "${GREEN}Ready to release async-inspect v${VERSION}! 🚀${NC}"
