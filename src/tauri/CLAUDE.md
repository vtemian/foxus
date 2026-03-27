# Rust Backend Rules

## Commands

```bash
make lint-rust        # clippy --all-targets -D warnings
make fmt-check        # cargo fmt --check
make check            # full pipeline (frontend + rust lint + fmt)
cargo fmt --all       # format all Rust code
cd src/tauri && cargo test  # run Rust tests
```

## Quality Gate

Enforced by `[lints]` in `Cargo.toml`. CI fails on any violation.

| Layer | Level | What |
|---|---|---|
| `clippy::pedantic` | deny | Full pedantic group as baseline |
| Pedantic overrides | allow | `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc` |
| `unwrap_used` | deny | Use `.expect("reason")` or `?` |
| `todo`, `dbg_macro` | deny | No dev artifacts in production |
| `print_stdout`, `print_stderr` | deny | Use `log` crate |
| `wildcard_enum_match_arm` | deny | Exhaustive matching |
| `indexing_slicing` | deny | Use `.get()`, `.first()`, slice patterns |
| `clone_on_ref_ptr` | deny | `Arc::clone(&x)` not `x.clone()` |
| `allow_attributes_without_reason` | deny | Document every `#[allow]` |
| `uninlined_format_args` | deny | `format!("{x}")` not `format!("{}", x)` |
| `as_conversions` | deny | Use `TryFrom`/`From`, not `as` casts |
| `panic` | deny | Return `Result`, let callers decide error handling |
| `unsafe_code` | deny | `#[expect]` with reason for justified FFI |

Complexity thresholds (clippy.toml): cognitive 30, function args 7, function lines 120, bool params 2.

Escape hatch: `#[expect(clippy::lint_name, reason = "...")]` per-site. Every suppression must have a reason.

## Error Handling

- Never `.unwrap()`. Use `.expect("reason")` for truly-unreachable cases. Use `?` for everything else
- Custom error types with `thiserror`. `AppError` for Tauri IPC
- Validate at construction time (parse, don't validate)

## Structure

- `commands/` - Tauri command handlers (IPC boundary)
- `models/` - Data models (Activity, Category, Rule, FocusSession, FocusSchedule)
- `db/` - Database connection, schema, migrations
- `platform/` - OS-specific window tracking (macOS via objc2, Linux via x11rb)
- `focus.rs` - Focus session and schedule management
- `tracker.rs` - Background activity polling
- `categorizer.rs` - Rule matching engine
- `native_host/` - Chrome extension native messaging
- `error.rs` - Custom error types
- `validation.rs` - Input validation

## Testing

- Test real behavior, not mocked behavior
- All error paths must have tests
- `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-dbg-in-tests` enabled in clippy.toml
