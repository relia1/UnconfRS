
# Thesis

## Overview

This project is a web application built using Rust, HTML, JavaScript, and Postgres. It assists in the organizing of an
unconference by allowing users to submit sessions, view a list of submitted sessions, vote on sessions they would like
to attend and allows an admin to create/update a schedule based on the submitted sessions.

## Quick Start

```sh
docker compose up --build
# Navigate to http://localhost:3000

```

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

## Technologies Used
- **Rust**: The programming language mainly used for the backend.
  - **SqlX**: Used for database operations. (Postgres)
  - **Axum**: Web framework used for routing and handling requests.
  - **Askama**: Template engine for rendering HTML.
- **HTML**: Used for structuring the web pages.
- **CSS**: Used for styling the web pages.
- **JavaScript**: Used for client-side scripting and handling form submissions.
- **DataTables**: jQuery plugin for enhancing HTML tables with advanced interaction controls.

### Running the Application Using Docker
1. Start the Rust server:
    ```sh
    docker compose up --build
    ```
2. Open your web browser and navigate to `http://localhost:3000`

### Running the Application Using Docker for Just Postgres

1. Start the database
    ```shell
   docker compose up --build db
   cargo run --bin unconfrs --release
    ```
2. Open your web browser and navigate to `http://localhost:3000`

## File Structure

- `sql_scripts`: Contains some helper sql scripts
- `src/`: Contains the Rust source code
  - `src/bin`: Contains the tool used for testing generated unconference's
    - `src/bin/test_unconf.rs`: The tool for generating an unconference
  - `src/controllers`: Contains the api controllers
  - `src/middleware`: Contains the web application middleware
  - `src/models`: Contains the api models
  - `src/routes`: Contains the routes for the web application
  - `src/types`: Contains some helpful types for the application
  - `src/api_docs.rs`: Contains OpenApi docs for the backend
  - `src/config.rs`: Contains the configuration of AppState and UnconfData
  - `src/db_config.rs`: Contains database configuration and connection
  - `src/lib.rs`: Making modules accessible to `src/bin/test_unconf.rs`
  - `src/main.rs`: Driver of the application
- `web/templates/`: Contains the HTML templates
  - `web/templates/snippets`: Contains reusable HTML snippets
- `web/scripts`: Contains JavaScript files
- `web/styles`: Contains CSS files
- `askama.toml`: Configuration file for Askama
- `README.md`: Project documentation
- `Cargo.toml`: Rust project configuration file
- `docker-compose.yml`: Docker containers configuration file
- `Dockerfile`: Docker configuration file
- `db/`: Contains the Postgres database password file
- `migrations/`: Contains the SQL migration files

## License
This project is licensed under the MIT License.