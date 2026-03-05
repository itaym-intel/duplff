# Skill: Add a Feature to duplff-core

Use when adding new functionality to the core library.

## Context

Read `.claude/docs/core-api.md` and `.claude/docs/architecture.md` for the full API and module layout.

## Steps

1. **Identify the module** — New types go in `models.rs`. New pipeline stages go in their own module. New file operations go in `actions.rs`. New storage goes in `log_store.rs` or `cache.rs`.

2. **Add the module** (if new):
   - Create `crates/duplff-core/src/{module}.rs`
   - Add `pub mod {module};` to `lib.rs`
   - Re-export from `lib.rs` if it's part of the public API

3. **Write tests first** — Every module has a `#[cfg(test)] mod tests` block at the bottom. Add tests before implementation. Use `tempfile::TempDir` for filesystem tests.

4. **Implement** — Follow existing patterns:
   - Return `Result<T>` using the crate's error type
   - Add new error variants to `DuplffError` if needed
   - All public types must derive `Serialize, Deserialize` if they cross crate boundaries
   - Use `rayon` for parallel work, `ignore` for directory walking

5. **Update the pipeline** (if needed) — If the feature affects the main `find_duplicates()` flow, modify `lib.rs` to integrate it.

6. **Verify**:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   cargo test -p duplff-core
   cargo clippy -p duplff-core -- -D warnings
   ```

7. **Update dependents** — If the core API changed, both CLI and GUI may need updates. Check if they still compile:
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

## Conventions

- All hashes are `[u8; 32]` (BLAKE3 output)
- Paths are always `PathBuf` in structs, `&Path` in function args
- Time is `SystemTime` internally, serialized as epoch seconds
- Keep `ProgressHandler` callbacks for any long-running operation
- Tests use `assert!` / `assert_eq!` — no test framework beyond std
