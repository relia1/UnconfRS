#!/bin/sh

# Use htpasswd to generate bcrypt hash
# Default cost is 12
# Default version is 2b
htpasswd -nBC 12 "" | tr -d ':'
