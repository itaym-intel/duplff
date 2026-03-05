# Skill: Troubleshoot CI Failures

Use when a GitHub Actions workflow fails and you need to diagnose and fix it.

## Diagnosis Steps

### 1. Get the failure details

```bash
# List recent runs
gh run list --limit 5

# View specific run
gh run view <run_id>

# View failed job logs
gh run view <run_id> --log-failed
```

### 2. Reproduce locally

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# Test job failures
cargo clippy --workspace -- -D warnings
cargo test --workspace

# Build-gui job failures
cd crates/duplff-gui
npm ci
npm run build
npx tauri build
```

### 3. Fix and push

Fix the issue, verify locally, commit, and push. The CI will re-run automatically on push to main.

## Common Failure Patterns

### Clippy Warnings

**Symptom**: `cargo clippy -- -D warnings` fails with unused variable, dead code, etc.

**Fix**: Address the warning. Common ones:
- Unused import → remove it
- Unused variable → prefix with `_` or remove
- Needless borrow → remove the `&`

### Test Failures

**Symptom**: `cargo test` fails on one or more tests.

**Fix**: Run the specific test locally with `--nocapture` to see output:
```bash
cargo test -p duplff-core failing_test_name -- --nocapture
```

### Missing System Dependencies

**Symptom**: Build fails with `pkg-config` errors or missing `.h` files.

**Fix**: Ensure CI installs the right packages. Current required list:
```
libwebkit2gtk-4.1-dev build-essential libssl-dev
libayatana-appindicator3-dev librsvg2-dev
```

### npm ci Failures

**Symptom**: `npm ci` fails with missing or mismatched packages.

**Fix**: Ensure `package-lock.json` is committed and up to date:
```bash
cd crates/duplff-gui
rm -rf node_modules
npm install
git add package-lock.json
```

### Tauri Build Failures

**Symptom**: `npx tauri build` fails.

**Checks**:
1. Is `npm run build` succeeding? (frontend must build first)
2. Is the `frontendDist` path correct in `tauri.conf.json`? (should be `"../build"`)
3. Are all Tauri plugins listed in both `Cargo.toml` and `tauri.conf.json` capabilities?

### Artifact Upload Failures

**Symptom**: Upload step fails or artifacts are empty.

**Fix**: Verify the artifact paths match what Tauri produces. Since this is a workspace, bundles go to `target/release/bundle/` (workspace root target dir), not inside `src-tauri/`.

## Workflow Files

- CI: `.github/workflows/ci.yml`
- Release: `.github/workflows/release.yml`

## Release-Specific Issues

### Tag Not Triggering Release

The release workflow only triggers on tags matching `v*`:
```bash
git tag v0.1.0
git push origin v0.1.0
```

### Missing Artifacts in Release

Check that the `files` glob in release.yml matches the actual output paths:
```yaml
files: |
  target/release/bundle/deb/*.deb
  target/release/bundle/appimage/*.AppImage
  target/release/duplff
```
