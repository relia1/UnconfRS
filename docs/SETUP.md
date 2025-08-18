# Setup & Installation

[<- Back to Main README](../README.md)

## Prerequisites

Before begining, make sure the following are installed:

- Rust and Cargo - [Install from rustup.rs](https://rustup.rs/)
- Docker - [Install Docker](https://docs.docker.com/get-started/get-docker/)
    - Alternatively: Local PostgreSQL if you prefer not to use Docker
- Database Password File: Create a directory named `db` in the project root with a `password.txt` file containing your
  PostgreSQL password

## Environment Setup

- If using docker, make sure the environment variables in compose.yaml are configured correctly
- If not using docker (or only using docker for part) make sure the environment variables in `.env` in the root
  directory are configured correctly

## Running options

### Option 1: Full Docker Setup

Simplest approach - Docker handles both the database and the application:

```sh
docker compose up --build
```

### Option 2: Docker for Just the Database

1. Start the database container
    ```sh
    docker compose up --build db
    ```
2. In a separate terminal, run the Rust application
    ```sh
    cargo run --bin unconfrs --release
    ```
3. Navigate to `http://localhost:3039`

## Troubleshooting

### Database Issues

If migration problems are encountered or during development the database gets in an unhealthy state, you might need to
reset the database completely

```sh
# Stop the containers
docker compose down

# Remove the database volume (this delete all the data)
docker volume rm unconfrs_db-data

# Start back up with fresh database
docker compose up --build
```

**Warning**: This will delete all existing data in the database.

## Common Issues

- Port Conflicts:
    - If port 3039 is in use, modify the port or stop the thing using port 3039
    - Database port may already be in use, you'll need to update the port in either `compose.yaml` or `.env` in the root
      directory of the project
- Missing `password.txt`: The application expects `db/password.txt` to exist

---

**Next Steps:**

- [User Guide](USER_GUIDE.md) - Learn how to use the application
- [Development Guide](DEVELOPMENT.md) - Development workflow and file structure
