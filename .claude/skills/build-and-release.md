# Skill: Build and Release

Use when building artifacts or creating a new release.

## Context

Read `.claude/docs/build-and-ci.md` for full build commands and CI pipeline details.

## Building Locally

### CLI Binary

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release -p duplff-cli
# Output: target/release/duplff
```

### GUI Application

```bash
# Prerequisites: system deps installed (see build-and-ci.md)
cd crates/duplff-gui
npm ci                  # install frontend deps
npm run build           # build frontend (SvelteKit → static)
npx tauri build         # build full app
# Output:
#   target/release/bundle/deb/duplff_*.deb
#   target/release/bundle/appimage/duplff_*.AppImage
```

### GUI Dev Mode

```bash
cd crates/duplff-gui && npx tauri dev
```

This runs the SvelteKit dev server on port 1420 and opens the Tauri window with hot reload.

## Creating a Release

### Pre-release Checklist

1. All tests pass: `cargo test --workspace`
2. Clippy clean: `cargo clippy --workspace -- -D warnings`
3. Frontend builds: `cd crates/duplff-gui && npm run build`
4. Version numbers updated if needed:
   - `crates/duplff-core/Cargo.toml` → `version`
   - `crates/duplff-cli/Cargo.toml` → `version`
   - `crates/duplff-gui/src-tauri/Cargo.toml` → `version`
   - `crates/duplff-gui/src-tauri/tauri.conf.json` → `version`
   - `crates/duplff-gui/package.json` → `version`
5. Changes committed and pushed to main

### Tag and Release

```bash
git tag v0.X.0
git push origin v0.X.0
```

This triggers `.github/workflows/release.yml` which:
1. Runs all tests
2. Builds GUI (.deb + .AppImage)
3. Builds CLI binary
4. Creates a GitHub Release with auto-generated notes
5. Attaches all 3 artifacts

### Monitoring the Release

```bash
gh run list --limit 3
gh run watch <run_id> --exit-status
gh release view v0.X.0 --json assets
```

## CI Troubleshooting

Common CI failures and fixes:

| Symptom | Cause | Fix |
|---------|-------|-----|
| Clippy fails | New warning in code | Fix the warning locally, push |
| Test fails | Flaky filesystem test | Check for missing `tempfile::TempDir` cleanup |
| System deps fail | Ubuntu version mismatch | Ensure `libwebkit2gtk-4.1-dev` (not 4.0) |
| npm ci fails | Missing package-lock.json | Ensure it's committed and tracked |
| Tauri build fails | Missing frontend build | Check `npm run build` runs before `npx tauri build` |
| Upload fails | Wrong artifact path | Bundle outputs are at `target/release/bundle/`, not `src-tauri/target/` |
