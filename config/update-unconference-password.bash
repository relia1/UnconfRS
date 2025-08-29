#!/bin/bash

# Displays how to use the script
usage() {
  echo "Usage: $0"
  echo "Updates the password for the unconference in the database"
  exit 1
}

# Check if database container is running
check_db_container() {
  if [ -z "$(docker ps -q -f name=unconfrs-db)" ]; then
    echo "Database container is not running"
    exit 1
  fi
}

# Update password in database
update_password() {
  local password="$1"

  check_db_container

  sql="DELETE FROM conference_password; INSERT INTO conference_password (password) VALUES ('$password')"
  docker exec -i unconfrs-db psql -d db -c "$sql"
}

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
hashed_pw=$(htpasswd -nBC 12 "" | tr -d ':')

update_password "$hashed_pw"
