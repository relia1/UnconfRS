> [!WARNING]
> Some documentation might not represent the current state of this repository.

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


### Quickstart (Docker)

* Build a Docker image

  ```sh
  docker build -t unconfrs --build-arg BUILD_TYPE=release .
  ```

* Run for the first time, configuring the database:

  ```sh
  docker run \
    -v unconfrs-data:/var/lib/postgresql/data \
    -e UNCONFERENCE_PASSWORD=fixme \
    -e ADMIN_EMAIL=fixme@example.org \
    -e ADMIN_PASSWORD=fixme \
    -p 127.0.0.1:3039:3039 \
    unconfrs
  ```

  This will create a new Docker volume for the conference database.

* Run subsequently, with the database configured:

  ```sh
  docker run \
    -v unconfrs-data:/var/lib/postgresql/data \
    -p 127.0.0.1:3039:3039 \
    unconfrs
  ```

## Production Deployment

### Nginx Configuration

To deploy with Nginx as a reverse proxy:

1. Copy the configuration file:
   ```bash
   sudo cp nginx-unconfrs.conf /etc/nginx/sites-available/unconfrs
   ```

1a. Set up SSL. Unconfrs should not be run without SSL
    protections, due to insecure password handling. We
    recommend the use of `certbot` / Let's Encrypt for this:
    the `nginx` configuration is set up to handle getting
    a certificate.

2. Update the configuration:
   - Replace `your-domain.com` with your actual domain name
   - Update SSL certificate paths in the configuration file

3. Enable the site:
   ```bash
   sudo ln -s /etc/nginx/sites-available/unconfrs /etc/nginx/sites-enabled/
   ```

4. Test and reload Nginx:
   ```bash
   sudo nginx -t && sudo systemctl reload nginx
   ```

### SSL Certificates

Configurations require SSL certificates. You can obtain them using:
- Let's Encrypt with Certbot
- Your certificate authority
- Self-signed certificates for testing

Update the certificate paths in the respective configuration files.


## License
This project is licensed under the MIT License.
