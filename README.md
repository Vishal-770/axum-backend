# Axum Rust Backend

A modular, production-ready Rust backend server built using the Axum web framework, SQLx for PostgreSQL database interactions, and bcrypt password hashing.

---

## Documentation Quick Links

- 🗄️ **[Database Management & Migrations](file:///home/vishal/Projects/axum-backend/database/README.md)**: Guide to running the Postgres container, managing database tables, using SQLx CLI migrations, and offline metadata setup.
- ⚠️ **[Error Handling Architecture](file:///home/vishal/Projects/axum-backend/src/errors/README.md)**: Deep dive into the custom `AppError` and `AuthError` enums, wrapping database errors securely, and HTTP status code mappings.
- 📦 **[DTOs & Request Validation](file:///home/vishal/Projects/axum-backend/src/dtos/README.md)**: Request payload validation mechanisms, field annotations, and separating internal DB models from API boundaries.

---

## Prerequisites

- **Rust**: Ensure the Rust toolchain (Cargo) is installed.
- **Docker**: Needed to run the PostgreSQL database locally.

---

## Quick Start

### 1. Start the Database
Spin up the local PostgreSQL database using Docker Compose:
```bash
docker compose -f database/docker-compose.yaml up -d
```
*Note: Details on logs, stopping, or accessing the Adminer Web UI can be found in the [Database Guide](file:///home/vishal/Projects/axum-backend/database/README.md).*

### 2. Configure Environment
Verify or create a `.env` file at the root of the project:
```env
DATABASE_URL=postgres://user:password@localhost:5432/mydb?sslmode=disable
```

### 3. Run the Server
Launch the Axum backend server:
```bash
cargo run
```
The server will automatically run any pending SQL migrations on boot and start listening on port `3000`.

---

## Testing API Endpoints

A Postman test collection is provided to make it easy to verify the routes:
- **Location**: [test/auth_tests.postman_collection.json](file:///home/vishal/Projects/axum-backend/test/auth_tests.postman_collection.json)
- **Supported Routes**:
  - `POST /auth/sign-up` (registers user accounts, case-insensitive)
  - `POST /auth/login` (logs in users)
  - `POST /auth/logout` (invalidates sessions)
  - `POST /auth/forgot-password` (initiates password reset)
  - `POST /auth/verify-email` (verifies user registration)
