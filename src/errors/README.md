# Error Handling System

This directory houses the error handling structure for the Axum backend. The project uses a hierarchical, domain-driven design powered by the `thiserror` crate to convert application errors into structured HTTP responses.

---

## The Core Concept: Why Error Wrapping?

In Rust, a function can only return **one** error type in its `Result<T, E>`. However, a service like `sign_up` can fail in two completely different ways:
1. **A Database Error** (e.g., query syntax error, connection timeout) $\rightarrow$ Produces `sqlx::Error`.
2. **An Authentication Error** (e.g., email already registered) $\rightarrow$ Produces `AuthError`.

To allow services to return both types of errors without boilerplate mapping on every line, we use the **Global Error Wrapper Pattern** via `AppError`. It acts as a container ("box") that holds either error.

```
                  ┌────────────── AppError ──────────────┐
                  │                                      │
                  │  Either:                             │
                  │  ┌──────────────┐  ┌──────────────┐  │
                  │  │  AuthError   │  │ sqlx::Error  │  │
                  │  └──────────────┘  └──────────────┘  │
                  │                                      │
                  └──────────────────────────────────────┘
```

---

## Technical Implementation Details

The magic of automatic error wrapping is powered by the `thiserror` crate's `#[from]` macro.

### 1. The Global Error Wrapper: `AppError` ([src/errors/mod.rs](file:///home/vishal/Projects/axum-backend/src/errors/mod.rs))

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    InternalServer,
}
```

Behind the scenes, `thiserror` expands `#[from]` to implement Rust's standard `From` trait. For example, it automatically generates:

```rust
// Generated automatically by #[from]
impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        AppError::Auth(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}
```

Because of these implementations, Rust's `?` operator and `.into()` method know exactly how to convert `AuthError` and `sqlx::Error` into `AppError` automatically.

---

### 2. Domain Errors: `AuthError` ([src/errors/auth_error.rs](file:///home/vishal/Projects/axum-backend/src/errors/auth_error.rs))

`AuthError` handles all client-facing authentication failures:

```rust
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("User already exists: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal server error")]
    InternalServer,
}
```

---

## How it Works in Practice (Step-by-Step)

### Scenario A: A Database Query Fails

1. **Failure**: Inside the service, `sqlx::query_as!(...).fetch_one(...)` fails (e.g. database went offline). This returns `Result<User, sqlx::Error>`.
2. **Propagation**: We append the `?` operator:
   ```rust
   let user = sqlx::query_as!(...).fetch_one(&mut *tx).await?;
   ```
3. **Conversion**: Rust sees that the query returns `sqlx::Error`, but the enclosing function returns `Result<_, AppError>`. Rust automatically calls `AppError::from(sqlx_error)`, wrapping it into `AppError::Database`.
4. **Handler Bubble-up**: The handler receives `AppError::Database`, maps it with `?` to propagate it out of the route.
5. **HTTP Response**: Axum calls `IntoResponse` for `AppError`. It logs the detailed database error internally but returns `500 Internal Server Error` with `{"error": "Internal database error"}` to the client (securing database details).

---

### Scenario B: Email is Already Registered

1. **Failure**: Inside the service, we detect that the user is already verified:
   ```rust
   if user.verified {
       return Err(AuthError::Conflict("User already exists".to_string()).into());
   }
   ```
2. **Conversion**: We instantiate `AuthError::Conflict` and call `.into()`. Since `From<AuthError>` is implemented for `AppError`, this converts it into `AppError::Auth(AuthError::Conflict)`.
3. **HTTP Response**: The handler propagates it using `?`. Axum calls `IntoResponse` which delegates to `AuthError`'s `IntoResponse`, returning:
   - **HTTP Status**: `409 Conflict`
   - **JSON Body**: `{"error": "User already exists"}`

---

## Response Mapping Matrix

| Error Type / Variant | HTTP Status Code | Client Payload | Internals Logged? |
|---|---|---|---|
| `AuthError::Conflict(msg)` | `409 Conflict` | `{"error": "<msg>"}` | No |
| `AuthError::Validation(msg)` | `400 Bad Request` | `{"error": "<msg>"}` | No |
| `AuthError::Unauthorized` | `401 Unauthorized` | `{"error": "Unauthorized"}` | No |
| `AppError::Database(sqlx_err)` | `500 Internal Server Error` | `{"error": "Internal database error"}` | **Yes (Detailed logs)** |
| `AppError::InternalServer` | `500 Internal Server Error` | `{"error": "Internal server error"}` | Yes |
