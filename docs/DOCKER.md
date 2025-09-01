# Docker Single-Container Deployment

This setup runs the entire application (web server + PostgreSQL) in a single container with persistent data storage.

## Quick Start

### Option 1: Docker Run (with persistent volume)
```bash
# First run - creates volume and initializes database
docker run \
  -v unconfrs-data:/var/lib/postgresql/data \
  -e UNCONFERENCE_PASSWORD=fixme \
  -e ADMIN_EMAIL=fixme@example.com \
  -e ADMIN_PASSWORD=fixme \
  -p 3039:3039 \
  unconfrs

# Subsequent runs - no environment variables needed
docker run \
  -v unconfrs-data:/var/lib/postgresql/data \
  -p 3039:3039 \
  unconfrs-single
```

### Option 2: Docker Compose
```bash
# Create .env file (optional - will use defaults if not provided)
echo "UNCONFERENCE_PASSWORD=conference2024" > .env
echo "ADMIN_EMAIL=admin@company.com" >> .env
echo "ADMIN_PASSWORD=securepass" >> .env

# Run with compose
docker-compose -f docker-compose.single.yaml up -d
```

## Environment Variables

### All Optional (only used on first run)
- `UNCONFERENCE_PASSWORD` - General site access password (default: `unconference123`)
- `ADMIN_EMAIL` - Admin user email (default: `admin@example.com`)  
- `ADMIN_PASSWORD` - Admin user password (default: `admin123`)
- `ADMIN_NAME` - Admin user display name (default: `Admin User`)

## Database Management

### Reset Database (Complete Wipe)
```bash
# Stop container
docker stop unconfrs-app

# Remove volume (THIS DELETES ALL DATA!)
docker volume rm unconfrs-data

# Start fresh
docker-compose -f docker-compose.single.yaml up -d
```

### Backup Database
```bash
# Backup to file (no password needed with trust auth)
docker exec unconfrs-app /usr/lib/postgresql/*/bin/pg_dump -h localhost -U postgres db > backup.sql

# Restore from file  
docker exec -i unconfrs-app /usr/lib/postgresql/*/bin/psql -h localhost -U postgres db < backup.sql
```

### Access Database Directly
```bash
# Connect to PostgreSQL in running container (no password needed)
docker exec -it unconfrs-app /usr/lib/postgresql/*/bin/psql -h localhost -U postgres db
```

## Application Access

- **Web Interface**: http://localhost:3039
- **Unconference Login**: Use `UNCONFERENCE_PASSWORD` value
- **Admin Login**: Use `ADMIN_EMAIL` and `ADMIN_PASSWORD` values

## Notes

- Database initialization only happens on first run (empty volume)
- Environment variables for unconference/admin are only needed initially
- Data persists across container restarts/updates when using volumes
- Container runs as non-root user for security
