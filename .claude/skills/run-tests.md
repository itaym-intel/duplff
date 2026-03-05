# Skill: Running and Writing Tests

Use when running tests, diagnosing failures, or writing new tests.

## Running Tests

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# Full workspace (51 tests)
cargo test --workspace

# By crate
cargo test -p duplff-core        # 41 unit tests
cargo test -p duplff-cli         # 4 unit tests
cargo test --test integration    # 6 integration tests (in duplff-core)

# Single test module
cargo test -p duplff-core scanner::tests
cargo test -p duplff-core actions::tests::dry_run_lists_files_to_delete

# With output
cargo test -p duplff-core -- --nocapture
```

## Test Locations

| Crate | Location | Count |
|-------|----------|-------|
| duplff-core | Inline `#[cfg(test)] mod tests` in each module | 41 |
| duplff-core | `crates/duplff-core/tests/integration.rs` | 6 |
| duplff-cli | Inline `#[cfg(test)] mod tests` in `format.rs` | 4 |

## Writing Core Tests

All core modules follow the same pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptive_name() {
        // Arrange
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("file.txt"), "content").unwrap();

        // Act
        let result = some_function(...);

        // Assert
        assert_eq!(result.field, expected);
    }
}
```

### Filesystem Test Helpers

- Use `tempfile::TempDir` for temporary directories (auto-cleaned)
- Create test files with `std::fs::write()`
- Use `SystemTime::UNIX_EPOCH` for deterministic mtimes in model construction

### Test Data Patterns

```rust
// DuplicateGroup for testing
fn make_group(paths: &[&str]) -> DuplicateGroup {
    DuplicateGroup {
        hash: [0u8; 32],
        size: 100,
        keep: RankedFile {
            entry: FileEntry { path: paths[0].into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            reason: KeepReason::LexicographicFirst,
        },
        duplicates: paths[1..].iter().map(|p| RankedFile {
            entry: FileEntry { path: (*p).into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            reason: KeepReason::LexicographicFirst,
        }).collect(),
    }
}
```

## Writing Integration Tests

Integration tests live in `crates/duplff-core/tests/integration.rs`. They test the full `find_duplicates()` pipeline:

```rust
#[test]
fn my_integration_test() {
    let dir = tempfile::TempDir::new().unwrap();
    // Create files with known content
    std::fs::write(dir.path().join("a.txt"), "same content").unwrap();
    std::fs::write(dir.path().join("b.txt"), "same content").unwrap();

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        ..Default::default()
    };

    let report = duplff_core::find_duplicates(&config, &duplff_core::progress::NoopProgress).unwrap();
    assert_eq!(report.groups.len(), 1);
}
```

## Frontend Build Verification

The GUI has no Svelte/TS tests, but the build itself validates types:

```bash
cd crates/duplff-gui && npm run build
```

A11y warnings from Svelte are non-blocking and can be suppressed with `<!-- svelte-ignore -->` comments.

## Clippy

Always run clippy alongside tests:

```bash
cargo clippy --workspace -- -D warnings
```

This catches warnings that tests don't.
