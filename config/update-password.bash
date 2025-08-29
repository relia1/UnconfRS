#!/bin/bash

# Displays how to use the script
usage() {
  echo "Usage: $0 <email>"
  echo "Updates the password for the specified email in the database"
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
  local email="$1"
  local password="$2"

  check_db_container

  sql="UPDATE users SET password = '$password' WHERE email = '$email'"
  docker exec -i unconfrs-db psql -d db -c "$sql"
}

# Make sure email was provided
if [ $# -ne 1 ]; then
  usage
fi

email="$1"

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
hashed_pw=$(htpasswd -nBC 12 "" | tr -d ':')

update_password "$email" "$hashed_pw"
