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
