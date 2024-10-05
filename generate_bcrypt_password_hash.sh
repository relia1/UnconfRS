#!/bin/bash

# Prompt user for a password
read -r -s -p "Enter Password: " password
echo

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
htpasswd -bnBC 12 "" "$password" | tr -d ':'