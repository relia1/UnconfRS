#!/bin/bash

# Displays how to use the script
usage() {
    echo -e "Usage: $0 <first_name> <last_name> <email> <role>\n"
    echo "Creates a new user with specified role"
    echo -e "Valid roles are 'user', 'facilitator', or 'admin'\n"
    exit 1
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
    local fname="$1"
    local lname="$2"
    local email="$3"
    local role="$4"
    local password="$5"

    check_db_container

    sql="INSERT INTO users (fname, lname, email, password) VALUES ('$fname', '$lname', '$email', '$password')"
    docker exec -i unconfrs-db-1 psql -d db -c "$sql"
    sql="INSERT INTO users_groups (user_id, group_id) VALUES ((SELECT id FROM users WHERE email = '$email'), (SELECT id FROM groups WHERE name = '$role'))"
    docker exec -i unconfrs-db-1 psql -d db -c "$sql"
}

# Make sure correct number of args were provided
if [ $# -ne 4 ]; then
    usage
fi

fname="$1"
lname="$2"
email="$3"
role="$4"

if [ "$role" != "user" ] && [ "$role" != "facilitator" ] && [ "$role" != "admin" ]; then
    usage
fi

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
hashed_pw=$(htpasswd -nBC 12 "" | tr -d ':')

create_user "$fname" "$lname" "$email" "$role" "$hashed_pw"
