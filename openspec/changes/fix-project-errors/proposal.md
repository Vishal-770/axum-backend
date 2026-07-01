## Why

`cargo clippy` surfaces 5 warnings-as-issues across the codebase — needless double-borrows in service functions and a missing `Default` impl on `MailService`. These are not hard compiler errors but they degrade code quality, will fail a stricter CI pipeline, and make the intent of the code harder to read.

## What Changes

- Remove unnecessary `&` borrow operators in `login_service.rs` where `access_secret` and `refresh_secret` are already `&String` references being passed to functions that expect `&str` — the double-borrow is a no-op the compiler silently eliminates.
- Same fix in `refresh_service.rs` for the same two variable sites.
- Add a `Default` implementation on `MailService` in `mail_config.rs` delegating to `Self::new()`, satisfying the `new_without_default` lint.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
<!-- No spec-level behaviour changes — these are purely internal Rust code quality fixes. -->

## Impact

- `src/config/mail_config.rs` — add `Default` impl
- `src/v1/auth/services/login_service.rs` — remove two needless borrows
- `src/v1/auth/services/refresh_service.rs` — remove two needless borrows
- No API surface, database schema, or runtime behaviour changes
