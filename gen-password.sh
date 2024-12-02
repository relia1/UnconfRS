#!/bin/sh

# Check if database container is running
check_db_container() {
  if [ -z "$(docker ps -q -f name=unconfrs-db-1)" ]; then
    echo "Database container is not running"
    exit 1
  fi
}

# Update password in database
update_password() {
  check_db_container
  sql="INSERT into users (username, password) VALUES ('admin', '$1')
  ON CONFLICT (username) DO UPDATE SET password = '$1'"
  docker exec -i unconfrs-db-1 psql -d db -c "$sql"
}

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
hashed_pw=$(htpasswd -nBC 12 "" | tr -d ':')
update_password "$hashed_pw"
