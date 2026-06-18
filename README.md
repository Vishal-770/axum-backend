# Axum Rust Backend

A Rust backend server built with Axum, SQLx (Postgres), and bcrypt.

---

## Database Management (Docker Compose)

The project includes a Docker Compose setup in the `database/` directory containing a **PostgreSQL** database and the **Adminer** database management UI.

### Prerequisites

Before starting, ensure that the Docker daemon is running. On Fedora/Linux:
- **System Service**: `sudo systemctl start docker`
- **Docker Desktop (User Service)**: `systemctl --user start docker-desktop`

---

### 1. Check Status (Is DB running or not?)

To check if the database and Adminer containers are currently running:

```bash
docker compose -f database/docker-compose.yaml ps
```

- **Running**: Containers will show a status of `Up`.
- **Not Running**: The command will show an empty list or `Exited` status.

---

### 2. Run / Start the Database

To start the database services in the background (detached mode):

```bash
docker compose -f database/docker-compose.yaml up -d
```

To view logs as they start or run:

```bash
docker compose -f database/docker-compose.yaml logs -f
```

---

### 3. Stop the Database

To stop the database services (containers are stopped but data is preserved):

```bash
docker compose -f database/docker-compose.yaml stop
```

To stop and completely remove the containers and network:

```bash
docker compose -f database/docker-compose.yaml down
```

---

### 4. Data Persistence

The PostgreSQL data is persisted using a Docker named volume called `pgdata`. 

- **Volume Location**: Configured in `database/docker-compose.yaml`.
- **Preserving Data**: Data is automatically persisted across container restarts, stops, and even when running `docker compose down`.
- **Wiping Data**: If you want to completely reset the database and delete all saved data, use the `-v` flag when bringing down the containers:
  ```bash
  docker compose -f database/docker-compose.yaml down -v
  ```

---

### 5. Database UI (Adminer)

Adminer provides a web interface to inspect and query the database.

- **URL**: [http://localhost:8080](http://localhost:8080)
- **Login Credentials**:
  - **System**: `PostgreSQL`
  - **Server**: `db` (or `localhost` if connecting from outside Docker network)
  - **Username**: `user`
  - **Password**: `password`
  - **Database**: `mydb`

---

## Database Migrations (Modifying Tables from Code)

This project uses SQLx's built-in migration system to manage the database schema directly from the codebase. Migrations are executed automatically when the backend server starts.

### How it Works

1. **Migration Files**: Stored in the [database/migrations](file:///home/vishal/Projects/axum-backend/database/migrations) directory.
2. **Naming Convention**: File names must be prefixed with a unique timestamp (e.g., `YYYYMMDDHHMMSS_action_name.sql`).
3. **Execution**: The server executes pending migrations on startup:
   ```rust
   sqlx::migrate!("./database/migrations").run(&pool).await;
   ```

---

### 1. Create a New Table

To create a new table, create a new SQL file in the migrations folder:
- **Example Filename**: `20260618000000_create_posts.sql`
- **File Content**:
  ```sql
  CREATE TABLE posts (
      id UUID PRIMARY KEY,
      title TEXT NOT NULL,
      body TEXT NOT NULL,
      user_id UUID REFERENCES users(id) ON DELETE CASCADE,
      created_at TIMESTAMP NOT NULL DEFAULT NOW()
  );
  ```

### 2. Edit an Existing Table (Alter Schema)

To edit a table (e.g., add, rename, or drop columns), create a new migration. **Never modify existing migration files** that have already been run.
- **Example Filename**: `20260618000001_add_bio_to_users.sql`
- **File Content**:
  ```sql
  -- Add a bio column to the users table
  ALTER TABLE users ADD COLUMN bio TEXT;
  ```

### 3. Delete a Table

To delete (drop) a table:
- **Example Filename**: `20260618000002_drop_posts.sql`
- **File Content**:
  ```sql
  DROP TABLE IF EXISTS posts;
  ```

---

### Key Rules for Migrations
- **Do not edit already applied migrations**: Once a migration runs in production or locally, editing its file will cause checksum mismatches in SQLx. Always create a *new* migration file for any schema changes.
- **Run migrations compile check**: Because SQLx checks queries at compile time, you must run your migrations against your database before compiling the Rust project.

---

## Running the Backend Application

Once the database is running:

1. **Verify configuration**: Make sure you have a `.env` file containing:
   ```env
   DATABASE_URL=postgres://user:password@localhost:5432/mydb?sslmode=disable
   ```
2. **Compile and run**:
   ```bash
   cargo run
   ```
   *Note: Database migrations run automatically at startup when the application boots.*

---

## Important Commands Quick Reference

| Command | Action |
|---|---|
| `docker compose -f database/docker-compose.yaml up -d` | Start database & UI in background |
| `docker compose -f database/docker-compose.yaml ps` | Check container status |
| `docker compose -f database/docker-compose.yaml logs -f` | Tail database logs |
| `docker compose -f database/docker-compose.yaml stop` | Stop database containers |
| `docker compose -f database/docker-compose.yaml down` | Remove containers & networks |
| `docker compose -f database/docker-compose.yaml down -v` | Remove containers, networks, & **wipe database data** |
