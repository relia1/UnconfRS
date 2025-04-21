#!/bin/bash

# Displays how to use the script
usage() {
  echo "Usage: $0 <username> <role>"
  echo "Creates a new user with specified role"
  echo "Valid roles are 'user', 'facilitator', or 'admin'"
}
# Check if database container is running
check_db_container() {
  if [ -z "$(docker ps -q -f name=unconfrs-db-1)" ]; then
    echo "Database container is not running"
    exit 1
  fi
}

# Update password in database
create_user() {
  local username="$1"
  local role="$2"
  local password="$3"

  check_db_container

  sql="INSERT INTO users (username, password) VALUES ('$username', '$password')"
  docker exec -i unconfrs-db-1 psql -d db -c "$sql"
  sql="INSERT INTO users_groups (user_id, group_id) VALUES ((SELECT id FROM users WHERE username = '$username'), (SELECT id FROM groups WHERE name = '$role'))"
  docker exec -i unconfrs-db-1 psql -d db -c "$sql"
}

# Make sure username was provided
if [ $# -ne 2 ]; then
  usage
fi

username="$1"
role="$2"

if [ "$role" != "user" ] && [ "$role" != "facilitator" ] && [ "$role" != "admin" ]; then
  usage
fi

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
hashed_pw=$(htpasswd -nBC 12 "" | tr -d ':')

create_user "$username" "$role" "$hashed_pw"
