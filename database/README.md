# Database Management & Migrations Guide

This directory houses the PostgreSQL database Docker setup and SQL database schema migrations for the Axum backend.

---

## 1. Database & UI Setup (Docker Compose)

The database services are managed using Docker Compose, which starts both a **PostgreSQL** database and an **Adminer** database client UI.

### Prerequisites
Before starting the database, ensure your Docker daemon is active:
- **Linux System Service**: `sudo systemctl start docker`
- **Docker Desktop (User Service)**: `systemctl --user start docker-desktop`

### Port Mappings
- **Postgres Database**: `5432`
- **Adminer Web Client**: `8080`

---

## 2. Docker Compose Commands Reference

All commands should be run from the project root directory.

### Start the Database (in background)
```bash
docker compose -f database/docker-compose.yaml up -d
```

### Check Database Status
To check if the Postgres and Adminer containers are currently running:
```bash
docker compose -f database/docker-compose.yaml ps
```

### View Database Logs
To view logs or tail container output in real-time:
```bash
docker compose -f database/docker-compose.yaml logs -f
```

### Stop the Database (Containers kept)
Stops the containers without removing them or their network:
```bash
docker compose -f database/docker-compose.yaml stop
```

### Down the Database (Containers removed)
Stops and removes the containers and network, but **persists data**:
```bash
docker compose -f database/docker-compose.yaml down
```

### Wipe All Database Data
If you need to completely delete the Postgres volume and start with a fresh, empty database:
```bash
docker compose -f database/docker-compose.yaml down -v
```

---

## 3. Database Web UI (Adminer)

Adminer provides an easy-to-use web interface to inspect tables, insert data, and run raw SQL queries.

- **URL**: [http://localhost:8080](http://localhost:8080)
- **Login Fields**:
  - **System**: `PostgreSQL`
  - **Server**: `db` (when inside Docker) or `localhost` (when outside Docker)
  - **Username**: `user`
  - **Password**: `password`
  - **Database**: `mydb`

---

## 4. SQLx CLI Installation

To manage database operations, migrations, and offline compiles from your command line, install the `sqlx-cli` tool. Specifying PostgreSQL features saves a lot of compile time:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

---

## 5. SQLx Migration Commands

Migrations allow you to version control your database schema. They are SQL files located in [migrations](file:///home/vishal/Projects/axum-backend/migrations).

### Create a New Migration File
Creates a blank migration file prefixed with a YYYYMMDDHHMMSS timestamp:
```bash
sqlx migrate add <migration_name>
```
*Example:* `sqlx migrate add create_posts` will create `migrations/20260618123000_create_posts.sql`.

### Run All Pending Migrations
Applies all pending migrations to the local database configured in `.env` (`DATABASE_URL`):
```bash
sqlx migrate run
```
*Note: The Axum backend is also configured to run migrations automatically on startup.*

### Revert the Latest Migration
Rolls back the most recently applied migration block (runs the inverse SQL or undoes the changes):
```bash
sqlx migrate revert
```

### View Migration Status
Shows a list of applied and pending migrations:
```bash
sqlx migrate info
```

---

## 6. Offline Compiles & IDE Support (.sqlx)

SQLx checks your raw SQL queries against a live database during the Rust compilation phase (`cargo check` or `cargo build`). In environments like IDE background checkers (VS Code, RustRover, CLion) where a live database connection may not be present, SQLx can operate in **Offline Mode**.

### Preparing Offline Metadata
Whenever you add or modify a SQL query in your Rust code, run the following command to update the local [.sqlx/](file:///home/vishal/Projects/axum-backend/.sqlx) metadata folder:

```bash
cargo sqlx prepare
```

### How it Works
1. `cargo sqlx prepare` checks your queries against the database and saves the results in JSON format inside the [.sqlx/](file:///home/vishal/Projects/axum-backend/.sqlx) folder.
2. The [.sqlx/](file:///home/vishal/Projects/axum-backend/.sqlx) directory is checked into Git version control.
3. The Rust compiler uses this metadata to compile your app offline without needing a database connection.
4. In VS Code or other IDEs, this prevents background checking errors and formatting conflicts on save.
