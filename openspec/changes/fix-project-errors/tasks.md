## 1. Fix Needless Borrows in login_service.rs

- [x] 1.1 In `src/v1/auth/services/login_service.rs`, change `&access_secret` → `access_secret` in the `create_access_token(...)` call
- [x] 1.2 In `src/v1/auth/services/login_service.rs`, change `&refresh_secret` → `refresh_secret` in the `create_refresh_token(...)` call

## 2. Fix Needless Borrows in refresh_service.rs

- [x] 2.1 In `src/v1/auth/services/refresh_service.rs`, change `&access_secret` → `access_secret` in the `create_access_token(...)` call
- [x] 2.2 In `src/v1/auth/services/refresh_service.rs`, change `&refresh_secret` → `refresh_secret` in the `create_refresh_token(...)` call

## 3. Add Default Impl to MailService

- [x] 3.1 In `src/config/mail_config.rs`, add `impl Default for MailService` delegating to `Self::new()`

## 4. Verify

- [x] 4.1 Run `cargo clippy 2>&1` and confirm zero warnings remain (excluding the third-party `sqlx-postgres` future-incompat note)
- [x] 4.2 Run `cargo test` and confirm all integration tests still pass
- [x] 4.3 Commit and push changes
