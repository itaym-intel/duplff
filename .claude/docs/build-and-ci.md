# Building, Testing, CI/CD

## Prerequisites

```bash
# Rust
export PATH="$HOME/.cargo/bin:$PATH"

# System deps (for GUI/Tauri)
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev

# Node.js 20 (for GUI frontend)
```

## Common Commands

### Test

```bash
# All workspace tests (51 total)
export PATH="$HOME/.cargo/bin:$PATH" && cargo test --workspace

# Core only (41 tests)
cargo test -p duplff-core

# CLI only (4 tests)
cargo test -p duplff-cli

# Specific test module
cargo test -p duplff-core scanner::tests

# Integration tests
cargo test --test integration
```

### Lint

```bash
# Clippy (whole workspace)
export PATH="$HOME/.cargo/bin:$PATH" && cargo clippy --workspace -- -D warnings

# Single crate
cargo clippy -p duplff-gui -- -D warnings
```

### Build

```bash
# CLI (debug)
cargo build -p duplff-cli

# CLI (release)
cargo build --release -p duplff-cli
# Output: target/release/duplff

# GUI frontend only
cd crates/duplff-gui && npm run build

# GUI full Tauri build
cd crates/duplff-gui && npx tauri build
# Output: target/release/bundle/deb/*.deb
#         target/release/bundle/appimage/*.AppImage

# GUI dev mode
cd crates/duplff-gui && npx tauri dev
```

### Run

```bash
# CLI interactive TUI
cargo run -p duplff-cli -- ~/Documents ~/Downloads

# CLI non-interactive
cargo run -p duplff-cli -- --json ~/Documents
cargo run -p duplff-cli -- --csv ~/Documents
cargo run -p duplff-cli -- --dry-run ~/Documents
```

## CI Pipeline (`.github/workflows/ci.yml`)

Triggers on: push to main, pull requests to main.

**Job 1: test** (ubuntu-latest)
1. Install Rust stable + clippy
2. Cache cargo (registry, git, target)
3. Install system deps (webkit2gtk, etc.)
4. `cargo clippy --workspace -- -D warnings`
5. `cargo test --workspace`

**Job 2: build-gui** (ubuntu-latest, needs: test)
1. Install system deps, Rust, Node.js 20
2. `npm ci` (frontend deps)
3. `npm run build` (frontend)
4. `npx tauri build` (full app)
5. Upload .deb and .AppImage as artifacts

## Release Pipeline (`.github/workflows/release.yml`)

Triggers on: push of tags matching `v*`.

1. Install everything (same as CI)
2. Run all tests
3. Build GUI (`npx tauri build`)
4. Build CLI (`cargo build --release -p duplff-cli`)
5. Create GitHub Release via `softprops/action-gh-release@v2`
   - Attaches: .deb, .AppImage, CLI binary
   - Auto-generates release notes

### Creating a Release

```bash
git tag v0.2.0
git push origin v0.2.0
```

This triggers the release workflow automatically.

## Verification Checklist

Before pushing, always verify:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo clippy --workspace -- -D warnings    # no warnings
cargo test --workspace                      # all 51 tests pass
cd crates/duplff-gui && npm run build       # frontend builds
```
