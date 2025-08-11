#!/bin/bash

# Install UnconfRS systemd services for autostart
# Run this script with sudo privileges

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Installing UnconfRS systemd services...${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Get the actual user who called sudo
ACTUAL_USER=${SUDO_USER:-$USER}
ACTUAL_HOME=$(getent passwd "$ACTUAL_USER" | cut -d: -f6)

echo -e "${YELLOW}Installing services for user: $ACTUAL_USER${NC}"
echo -e "${YELLOW}Home directory: $ACTUAL_HOME${NC}"

# Create systemd directory if it doesn't exist
mkdir -p /etc/systemd/system

# Copy service files and update paths
echo -e "${YELLOW}Installing PostgreSQL service...${NC}"
sed "s|/home/bart/unconfrs|$ACTUAL_HOME/unconfrs|g" unconfrs-postgres.service > /etc/systemd/system/unconfrs-postgres.service

echo -e "${YELLOW}Installing application service...${NC}"
sed -e "s|/home/bart/unconfrs|$ACTUAL_HOME/unconfrs|g" \
    -e "s|User=bart|User=$ACTUAL_USER|g" \
    -e "s|Group=bart|Group=$ACTUAL_USER|g" \
    unconfrs-app.service > /etc/systemd/system/unconfrs-app.service

# Reload systemd
echo -e "${YELLOW}Reloading systemd daemon...${NC}"
systemctl daemon-reload

# Enable services
echo -e "${YELLOW}Enabling services for autostart...${NC}"
systemctl enable unconfrs-postgres.service
systemctl enable unconfrs-app.service

# Check if Docker is running and enabled
if ! systemctl is-active --quiet docker; then
    echo -e "${YELLOW}Starting Docker service...${NC}"
    systemctl start docker
fi

if ! systemctl is-enabled --quiet docker; then
    echo -e "${YELLOW}Enabling Docker for autostart...${NC}"
    systemctl enable docker
fi

# Add user to docker group if not already in it
if ! groups "$ACTUAL_USER" | grep -q docker; then
    echo -e "${YELLOW}Adding $ACTUAL_USER to docker group...${NC}"
    usermod -aG docker "$ACTUAL_USER"
    echo -e "${YELLOW}User added to docker group. You may need to log out and back in.${NC}"
fi

echo -e "${GREEN}Installation complete!${NC}"
echo -e "${GREEN}Services installed:${NC}"
echo "  - unconfrs-postgres.service (PostgreSQL database)"
echo "  - unconfrs-app.service (UnconfRS application)"
echo ""
echo -e "${GREEN}To start the services now:${NC}"
echo "  sudo systemctl start unconfrs-postgres"
echo "  sudo systemctl start unconfrs-app"
echo ""
echo -e "${GREEN}To check service status:${NC}"
echo "  sudo systemctl status unconfrs-postgres"
echo "  sudo systemctl status unconfrs-app"
echo ""
echo -e "${GREEN}To view logs:${NC}"
echo "  sudo journalctl -u unconfrs-postgres -f"
echo "  sudo journalctl -u unconfrs-app -f"
echo ""
echo -e "${YELLOW}Note: Make sure to build the application first:${NC}"
echo "  cd $ACTUAL_HOME/unconfrs"
echo "  cargo build --release --bin unconfrs"