
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

## License
This project is licensed under the MIT License.