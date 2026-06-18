# Data Transfer Objects (DTOs) & Request Validation

This directory contains the Data Transfer Objects (DTOs) used to define the request and response shapes at the boundaries of the API. It also handles request validation before payloads reach the core business services.

---

## Architectural Role

DTOs act as an entry and exit barrier for our application, separating the public HTTP API contract from the internal database models (`User`).

```
                    Client Request (JSON)
                             │
                             ▼
                    [ DTO / SignUpDto ]
                             │
             (1) Auto-deserialize via serde
             (2) Declarative validation via validator
                             │
            ┌────────────────┴────────────────┐
            ▼ (Valid)                         ▼ (Invalid)
    [ Handler Layer ]                 [ 400 Bad Request ]
            │
            ▼
     [ Service Layer ]
            │
            ▼
    [ Database / Model ]
```

---

## 1. Declarative DTO Definitions ([src/dtos/auth_dtos/mod.rs](file:///home/vishal/Projects/axum-backend/src/dtos/auth_dtos/mod.rs))

We use the `validator` crate with standard attribute macros to define request constraints directly on the struct fields.

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct SignUpDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 6, message = "Password must be at least 6 characters long"))]
    pub password: String,

    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub user_name: String,
}
```

### Key Attributes Used:
- `#[derive(Validate)]`: Generates the validation logic implementation automatically for the struct.
- `#[validate(email)]`: Ensures the string conforms to a valid email format.
- `#[validate(length(min = X, message = "..."))]`: Constraints the string length and sets a custom user-friendly error message if it fails.

---

## 2. Triggering Validation in Handlers ([src/handlers/auth/sign_up.rs](file:///home/vishal/Projects/axum-backend/src/handlers/auth/sign_up.rs))

Validation does not run automatically on deserialization. It must be triggered explicitly in the handler layer by calling `.validate()` on the payload:

```rust
pub async fn sign_up_handler(
    State(state): State<AppState>,
    Json(payload): Json<SignUpDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Explicitly validate the input
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call sign_up service if valid...
}
```

### The Error Mapping Flow:
1. `payload.validate()` returns a `Result<(), ValidationErrors>`.
2. `.map_err(|e| AuthError::Validation(e.to_string()))` catches any validation errors and maps them to our specific `AuthError::Validation` domain error.
3. The `?` operator converts `AuthError::Validation` into `AppError` and returns it early.
4. Axum's response layer converts this to an HTTP `400 Bad Request` with the JSON validation messages.

---

## 3. Response DTOs

Response DTOs (like `CreateUserResponse`) filter sensitive database columns (like `password` hashes) from being returned to the client:

```rust
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub verified: bool,
}
```
This guarantees that only public information is sent back to the API consumers.
