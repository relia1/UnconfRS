#!/bin/bash
set -e

# Set default unconference password and admin credentials
UNCONFERENCE_PASSWORD=${UNCONFERENCE_PASSWORD:-unconference123}
ADMIN_EMAIL=${ADMIN_EMAIL:-admin@example.com}
ADMIN_PASSWORD=${ADMIN_PASSWORD:-admin123}
ADMIN_NAME=${ADMIN_NAME:-Admin User}

# Initialize PostgreSQL if not already done
if [ ! -d "/var/lib/postgresql/data/base" ]; then
    echo "Initializing PostgreSQL database..."
    /usr/lib/postgresql/*/bin/initdb -D /var/lib/postgresql/data --auth-local=trust --auth-host=trust --username=appuser

    # Start PostgreSQL temporarily to set up database
    /usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data -l /var/lib/postgresql/data/logfile -w start

    # Create postgres user and database (no password needed with trust auth)
    /usr/lib/postgresql/*/bin/psql -h localhost -d template1 -c "CREATE USER postgres SUPERUSER;"
    /usr/lib/postgresql/*/bin/createdb -h localhost -O postgres db

    # Stop PostgreSQL
    /usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data -w stop
fi

# Start PostgreSQL and wait for it to be ready
echo "Starting PostgreSQL..."
/usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data -l /var/lib/postgresql/data/logfile -w start

# The application will handle password and admin user initialization automatically

# Set environment variables for the app (no password needed with trust auth)
export PG_USER=postgres
export PG_HOST=localhost
export PG_PORT=5432
export PG_DBNAME=db
export CONTAINER=true
export RUST_LOG=info
export SCRIPTS_DIR=/scripts
export STYLES_DIR=/styles
export SCHEDULING_METHOD=localsearch

# Export initialization variables so the app can use them
export UNCONFERENCE_PASSWORD
export ADMIN_EMAIL
export ADMIN_PASSWORD
export ADMIN_NAME

# Start the application
echo "Starting application..."
exec /bin/server
