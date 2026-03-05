# Skill: Add a Feature to duplff-gui

Use when adding new functionality to the GUI application.

## Context

Read `.claude/docs/gui-guide.md` for the full frontend/backend structure.

**IMPORTANT**: Do NOT modify duplff-core files unless the task explicitly requires core changes.

## Adding a Backend Command

1. **Define response types** in `crates/duplff-gui/src-tauri/src/commands.rs`:
   ```rust
   #[derive(Serialize)]
   pub struct MyResponse { pub field: String }
   ```

2. **Write the command**:
   ```rust
   #[tauri::command]
   pub fn my_command(state: State<'_, AppState>) -> Result<MyResponse, String> {
       // Use state.report.lock().unwrap() for report access
       // Use duplff_core::* for core logic
       Ok(MyResponse { field: "value".into() })
   }
   ```

3. **Register** in `lib.rs` — add `commands::my_command` to the `generate_handler![]` macro

4. **Verify backend**:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   cargo clippy -p duplff-gui -- -D warnings
   ```

## Wiring to Frontend

1. **Add TypeScript type** in `src/lib/types.ts` — mirror the Rust struct exactly (snake_case field names)

2. **Add API function** in `src/lib/api.ts`:
   ```typescript
   export async function myCommand(): Promise<MyResponse> {
     return invoke('my_command');
   }
   ```

3. **Add store** (if needed) in `src/lib/stores.ts`

4. **Use in screen/component** — import from `$lib/api` and call

## Adding a Screen

1. Create `src/lib/screens/MyScreen.svelte`
2. Add `'myscreen'` to the `Screen` type union in `types.ts`
3. Add the import and conditional render in `src/routes/+page.svelte`
4. Manage transitions via `currentScreen.set('myscreen')`

## Adding a Component

1. Create in `src/lib/components/MyComponent.svelte`
2. Use `export let` for props (Svelte 5 style)
3. Import and use in screens

## Styling Rules

- Use Tailwind utility classes only — no inline styles, no custom CSS
- Dark theme: `bg-gray-900` base, `bg-gray-800` for surfaces
- Custom colors: `text-keep` (green), `text-delete` (red), `text-active` (cyan) — defined in `app.css` `@theme`
- Text sizes: `text-xs` for UI elements, `text-sm` sparingly
- Font: `font-mono` for paths, numbers, hashes
- Spacing: compact — `py-1.5 px-2.5` for inputs, `gap-1.5` between elements
- Transitions: `transition-colors` on hover states
- **Avoid visual clutter** — no unnecessary borders, backgrounds, or decorations

## Verify

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo clippy -p duplff-gui -- -D warnings
cd crates/duplff-gui && npm run build
```

## Common Patterns

- **Toast notifications**: Set a `toast` variable, clear with `setTimeout` after 3s
- **Confirm dialogs**: Use `ConfirmDialog` component with `showConfirm` boolean
- **Event listeners**: Set up in `onMount`, clean up in `onDestroy` via `UnlistenFn[]`
- **Store updates after actions**: Call `getResults()` after trash/undo to refresh report
