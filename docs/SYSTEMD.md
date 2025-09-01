## System Autostart Configuration

To automatically start the application and PostgreSQL database on system boot:

### Prerequisites

Ensure you have:
- Docker installed and running
- Application built in release mode: `cargo build --release --bin unconfrs`
- Database password file created: `secrets/db-password.txt`

### Installation

1. Run the installation script:
   ```bash
   sudo ./install-services.sh
   ```

This script will:
- Install systemd services for both PostgreSQL and the application
- Configure proper service dependencies
- Enable autostart on system boot
- Add the current user to the docker group if needed

### Manual Installation

If you prefer manual setup:

1. Copy service files to systemd:
   ```bash
   sudo cp unconfrs-postgres.service /etc/systemd/system/
   sudo cp unconfrs-app.service /etc/systemd/system/
   ```

2. Update paths in service files if your installation directory differs from `/home/bart/unconfrs`

3. Enable services:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable unconfrs-postgres.service
   sudo systemctl enable unconfrs-app.service
   ```

### Service Management

Start services:
```bash
sudo systemctl start unconfrs-postgres
sudo systemctl start unconfrs-app
```

Check status:
```bash
sudo systemctl status unconfrs-postgres
sudo systemctl status unconfrs-app
```

View logs:
```bash
sudo journalctl -u unconfrs-postgres -f
sudo journalctl -u unconfrs-app -f
```

Stop services:
```bash
sudo systemctl stop unconfrs-app
sudo systemctl stop unconfrs-postgres
```

### Service Details

- **unconfrs-postgres.service**: Manages the PostgreSQL Docker container
- **unconfrs-app.service**: Manages the Rust application with proper dependencies

The application service will automatically wait for PostgreSQL to be ready before starting.
