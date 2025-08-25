#!/bin/bash
set -e

# Set default password if not provided
POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-changeme123}

# Write password to expected file location
mkdir -p /tmp
echo "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" > /tmp/db-password.txt

# Initialize PostgreSQL if not already done
if [ ! -d "/var/lib/postgresql/data/base" ]; then
    echo "Initializing PostgreSQL database..."
    /usr/lib/postgresql/*/bin/initdb -D /var/lib/postgresql/data --auth-local=trust --auth-host=trust --username=appuser
    
    # Start PostgreSQL temporarily to set up database
    /usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data -l /var/lib/postgresql/data/logfile start
    
    # Wait for PostgreSQL to start
    sleep 3
    
    # Create postgres user and database
    /usr/lib/postgresql/*/bin/psql -h localhost -d template1 -c "CREATE USER postgres SUPERUSER PASSWORD '${POSTGRES_PASSWORD}';"
    /usr/lib/postgresql/*/bin/createdb -h localhost -O postgres db
    
    # Update pg_hba.conf to require password for host connections
    echo "host all all 0.0.0.0/0 md5" >> /var/lib/postgresql/data/pg_hba.conf
    
    # Stop PostgreSQL
    /usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data stop
fi

# Start PostgreSQL
echo "Starting PostgreSQL..."
/usr/lib/postgresql/*/bin/pg_ctl -D /var/lib/postgresql/data -l /var/lib/postgresql/data/logfile start

# Wait for PostgreSQL to be ready
sleep 3

# Set environment variables for the app
export PG_USER=postgres
export PG_PASSWORDFILE=/tmp/db-password.txt
export PG_HOST=localhost
export PG_PORT=5432
export PG_DBNAME=db
export CONTAINER=true
export RUST_LOG=info
export SCRIPTS_DIR=/scripts
export STYLES_DIR=/styles

# Start the application
echo "Starting application..."
exec /bin/server