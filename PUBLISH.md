# Publishing Guide

Steps to publish a new version of spacetime_tiled to crates.io.

## Prerequisites

1. **Get a crates.io account**
   - Go to https://crates.io/
   - Sign in with GitHub

2. **Generate an API token**
   - Go to https://crates.io/settings/tokens
   - Click "New Token"
   - Give it a name like "spacetime_tiled-publish"
   - Copy the token

3. **Add token to GitHub secrets**
   - Go to your repo: Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `CARGO_TOKEN`
   - Value: paste your crates.io token

## Publishing Process

### 1. Update Version

Edit `Cargo.toml`:
```toml
version = "0.2.0"  # or whatever the new version is
```

### 2. Update CHANGELOG.md

Add a new section for the version:
```markdown
## [0.2.0] - 2025-XX-XX

### Added
- New feature

### Fixed
- Bug fix

### Changed
- Breaking change
```

### 3. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "Bump version to 0.2.0"
git push
```

### 4. Create and Push Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 5. Wait for GitHub Actions

The release workflow will automatically:
- Run tests
- Build the package
- Publish to crates.io
- Create a GitHub release

Check progress: https://github.com/jbuehler23/spacetime_tiled/actions

## Manual Publishing (if needed)

If the automatic workflow fails:

```bash
# Verify everything builds
cargo check
cargo test --lib
cargo package --no-verify

# Publish manually
cargo login
cargo publish
```

## After Publishing

1. **Verify on crates.io**: https://crates.io/crates/spacetime_tiled
2. **Check docs.rs**: https://docs.rs/spacetime_tiled (builds automatically)
3. **Announce**: Update any discussions, Discord, etc.

## Troubleshooting

### "crate already exists"
Someone else published with this name first, or you already published this version. Bump the version and try again.

### Build fails in CI
Check the Actions logs. Common issues:
- Missing LLVM/clang (should be in workflow)
- Test failures
- Cargo.lock out of sync

### docs.rs build fails
Check https://docs.rs/crate/spacetime_tiled/latest/builds

Usually it's the same LLVM/clang issue. The docs.rs team can help if it persists.

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **0.x.y** - Pre-1.0, API can change anytime
- **x.0.0** - Breaking changes
- **x.y.0** - New features, no breaking changes
- **x.y.z** - Bug fixes only
