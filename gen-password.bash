#!/bin/bash

# Displays how to use the script
usage() {
  echo "Usage: $0 <username>"
  echo "Updates or sets the password for the specified username in the database"
}
# Check if database container is running
check_db_container() {
  if [ -z "$(docker ps -q -f name=unconfrs-db-1)" ]; then
    echo "Database container is not running"
    exit 1
  fi
}

# Update password in database
update_password() {
  local username="$1"
  local password="$2"

  check_db_container

  sql="INSERT into users (username, password) VALUES ('$username', '$password')
  ON CONFLICT (username) DO UPDATE SET password = '$1'"
  docker exec -i unconfrs-db-1 psql -d db -c "$sql"
}

# Make sure username was provided
if [ $# -ne 1 ]; then
  usage
fi

username="$1"

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
hashed_pw=$(htpasswd -nBC 12 "" | tr -d ':')

update_password "$username" "$hashed_pw"
