# unconfrs: Unconference Webapp in Rust
Robert Elia 2025

This project is a web application built using Rust, HTML,
JavaScript, and Postgres. It assists in the organizing of an
unconference by allowing users to submit sessions, view a
list of submitted sessions, vote on sessions they would like
to attend and allows an admin to create/update a schedule
based on the submitted sessions.

## Quick Links

- [Setup & Installation](docs/SETUP.md)
- [User Guide](docs/USER_GUIDE.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Architecture & Technologies](docs/ARCHITECTURE.md)

## Features

- Session Creation: Submit a new session with title and content details
- Interactive Sessions Table: View a list of submitted sessions in a table with editing, voting, deletion, sorting, and
  searching functionalities.
- Schedule Creation: Initialize empty schedule and populate with submitted sessions
- Drag & Drop: Manually adjust the schedule with drag and drop
- Authentication and User Roles: Different levels of control/access with user, facilitator, and admin roles.


### Running the Application Using Docker

1. Start the Rust server:
    ```sh
    docker compose up --build
    ```
2. Open your web browser and navigate to `http://localhost:3039`

### Running the Application Using Docker for Just Postgres

1. Start the database
    ```shell
   docker compose up --build db
   cargo run --bin unconfrs --release
    ```
2. Open your web browser and navigate to `http://localhost:3039`

## Production Deployment

### Apache2 Configuration

To deploy with Apache2 as a reverse proxy:

1. Copy the configuration file:
   ```bash
   sudo cp apache2-unconfrs.conf /etc/apache2/sites-available/
   ```

2. Enable required modules:
   ```bash
   sudo a2enmod ssl proxy proxy_http headers rewrite
   ```

3. Update the configuration:
   - Replace `your-domain.com` with your actual domain name
   - Update SSL certificate paths in the configuration file

4. Enable the site and reload Apache:
   ```bash
   sudo a2ensite apache2-unconfrs.conf
   sudo systemctl reload apache2
   ```

### Nginx Configuration

To deploy with Nginx as a reverse proxy:

1. Copy the configuration file:
   ```bash
   sudo cp nginx-unconfrs.conf /etc/nginx/sites-available/unconfrs
   ```

2. Enable the site:
   ```bash
   sudo ln -s /etc/nginx/sites-available/unconfrs /etc/nginx/sites-enabled/
   ```

3. Update the configuration:
   - Replace `your-domain.com` with your actual domain name
   - Update SSL certificate paths in the configuration file

4. Test and reload Nginx:
   ```bash
   sudo nginx -t && sudo systemctl reload nginx
   ```

### SSL Certificates

Both configurations require SSL certificates. You can obtain them using:
- Let's Encrypt with Certbot
- Your certificate authority
- Self-signed certificates for testing

Update the certificate paths in the respective configuration files.

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

## License
This project is licensed under the MIT License.
