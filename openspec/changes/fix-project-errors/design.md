## Context

`cargo clippy` identifies 5 warnings across the project:

1. **`needless_borrow` × 4** — In `login_service.rs` and `refresh_service.rs`, variables `access_secret` and `refresh_secret` are already `&String` (references into `AppState.config`). Passing `&access_secret` (a `&&String`) to functions that accept `&str` creates a redundant double-reference that the compiler silently auto-derefs. Clippy flags this because it adds visual noise without purpose.

2. **`new_without_default` × 1** — `MailService` has a `new()` constructor but no `impl Default`. Rust convention (and Clippy's `new_without_default` lint) requires that if a `new()` takes no arguments and returns `Self`, a `Default` impl should be present so the type can be used with `Default::default()` and trait-bounded generic code.

There are no hard compiler errors (all code compiles). These are lint warnings that should be cleaned up.

## Goals / Non-Goals

**Goals:**
- Achieve zero Clippy warnings under the default lint set
- Follow Rust idioms for reference passing and the `Default` trait

**Non-Goals:**
- Upgrading the `sqlx-postgres` future-incompatibility warning (third-party crate, not our code)
- Changing any runtime behaviour, API, or database schema

## Decisions

### Fix `needless_borrow` by removing `&` prefixes
The variables are already `&String`. Functions like `create_access_token` and `create_refresh_token` accept `&str`, and `&String` coerces to `&str` automatically. Adding another `&` produces `&&String`, which also coerces but unnecessarily. Remove the superfluous `&`.

**Files affected:**
- `src/v1/auth/services/login_service.rs` — lines calling `create_access_token` and `create_refresh_token`
- `src/v1/auth/services/refresh_service.rs` — same two call sites

### Fix `new_without_default` by adding a `Default` impl
Add a trivial blanket delegation:
```rust
impl Default for MailService {
    fn default() -> Self {
        Self::new()
    }
}
```
**File affected:** `src/config/mail_config.rs`

## Risks / Trade-offs

- **Zero runtime risk** — all changes are syntactic. No logic path changes.
- **No breaking changes** — `Default` is an additive trait impl; removing `&` does not change function call semantics.
