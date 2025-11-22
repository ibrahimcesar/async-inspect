# Release Checklist

Use this checklist when preparing a new release of async-inspect.

## Pre-Release (1-2 weeks before)

### Code Quality
- [ ] All CI checks passing on main branch
- [ ] No critical or high-priority bugs in issue tracker
- [ ] Code coverage above 70%
- [ ] All examples run successfully
- [ ] Performance benchmarks show no regressions

### Documentation
- [ ] README.md is up-to-date
- [ ] CHANGELOG.md has entry for this release
- [ ] API documentation is complete (`cargo doc --all-features`)
- [ ] All public APIs have doc comments
- [ ] Examples are documented and working
- [ ] Migration guide written (if breaking changes)

### Testing
- [ ] Manual testing on Linux
- [ ] Manual testing on macOS
- [ ] Manual testing on Windows
- [ ] Test with Tokio (latest version)
- [ ] Test with different feature combinations
- [ ] Test all examples
- [ ] Integration tests pass

### Dependencies
- [ ] All dependencies are up-to-date
- [ ] No known security vulnerabilities (`cargo audit`)
- [ ] Dependency licenses are compatible
- [ ] Unnecessary dependencies removed

## Version Update

### Version Numbers
- [ ] Update version in `Cargo.toml`
- [ ] Update version in `async-inspect-macros/Cargo.toml`
- [ ] Update version in documentation
- [ ] Update version in README examples
- [ ] Ensure versions follow semantic versioning

### Changelog
- [ ] Move `[Unreleased]` items to new version section
- [ ] Add release date
- [ ] Include all notable changes
- [ ] Categorize changes (Added, Changed, Deprecated, Removed, Fixed, Security)
- [ ] Add GitHub compare links

## Release Process

### Pre-Flight Checks
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-features -- -D warnings`
- [ ] `cargo test --all-features`
- [ ] `cargo build --release --all-features`
- [ ] `cargo doc --all-features --no-deps`

### Git Operations
- [ ] Commit all changes: `git commit -m "chore: prepare vX.Y.Z release"`
- [ ] Push to main: `git push origin main`
- [ ] Wait for CI to pass
- [ ] Create and push tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
- [ ] Push tag: `git push origin vX.Y.Z`

### Automated (via GitHub Actions)
- [ ] GitHub release created automatically
- [ ] Binaries built for all platforms
- [ ] Published to crates.io automatically
- [ ] Docker image built and published

### Manual Steps
- [ ] Verify crates.io listing: https://crates.io/crates/async-inspect
- [ ] Verify GitHub release: https://github.com/ibrahimcesar/async-inspect/releases
- [ ] Verify Docker image: https://hub.docker.com/r/USERNAME/async-inspect
- [ ] Test installation: `cargo install async-inspect`

## Post-Release

### Communication
- [ ] Announce on Reddit r/rust
- [ ] Post on Hacker News
- [ ] Tweet about release
- [ ] Update Discord/Slack community
- [ ] Blog post (if major release)

### Documentation
- [ ] Verify docs.rs build: https://docs.rs/async-inspect
- [ ] Update GitHub Pages documentation
- [ ] Update examples repository (if separate)

### Monitoring
- [ ] Monitor issue tracker for new bugs
- [ ] Monitor crates.io download stats
- [ ] Check for new security advisories
- [ ] Review user feedback

### Cleanup
- [ ] Create new `[Unreleased]` section in CHANGELOG
- [ ] Close milestone for this release (if using milestones)
- [ ] Plan next release
- [ ] Archive old release notes (if applicable)

## Rollback Plan (If Issues Found)

### Immediate Actions
- [ ] Identify the issue severity
- [ ] Yank problematic version from crates.io: `cargo yank --vers X.Y.Z`
- [ ] Create hotfix branch if needed
- [ ] Document the issue in GitHub

### Communication
- [ ] Post notice on GitHub release
- [ ] Update Discord/community channels
- [ ] Create issue for tracking

### Fix and Re-release
- [ ] Fix the issue
- [ ] Increment patch version (X.Y.Z+1)
- [ ] Follow release checklist again
- [ ] Un-yank if appropriate, or publish new version

## Release Types

### Patch Release (X.Y.Z)
- Bug fixes only
- No breaking changes
- No new features
- Can be released quickly

### Minor Release (X.Y.0)
- New features
- No breaking changes
- Deprecations allowed
- More extensive testing needed

### Major Release (X.0.0)
- Breaking changes allowed
- Complete migration guide required
- Beta/RC cycle recommended
- Extended testing period

## Notes

- Always use semantic versioning
- Keep CHANGELOG.md updated
- Test on multiple platforms
- Communicate breaking changes clearly
- Be responsive to issues after release

## Quick Release Commands

```bash
# Update version
vim Cargo.toml
vim async-inspect-macros/Cargo.toml
vim CHANGELOG.md

# Pre-flight checks
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo build --release --all-features

# Commit and tag
git add .
git commit -m "chore: prepare vX.Y.Z release"
git push origin main
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z

# Manual publish (if automation fails)
cd async-inspect-macros && cargo publish
cd .. && cargo publish
```
