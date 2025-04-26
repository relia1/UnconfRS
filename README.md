
# Thesis

## Overview
This project is a web application built using Rust, HTML, JavaScript, and Postgres. It assists in
the organizing of an unconference by allowing users to submit topics, view a list of submitted topics,
and create/update a schedule based on the submitted topics. 

## Features
- Submit a new topic with details such as title, content, and presenter's name, email, and phone 
  number.
- View a list of submitted topics in a table with deletion, sorting, searching, and pagination 
  functionalities.
- Initialize schedule based on submitted rooms
- Populate schedule with submitted topics
- Manually drag and drop topics to adjust the schedule

## Technologies Used
- **Rust**: The programming language mainly used for the backend.
  - **SqlX**: Used for database operations. (Postgres)
  - **Axum**: Web framework used for routing and handling requests.
  - **Askama**: Template engine for rendering HTML.
- **HTML**: Used for structuring the web pages.
- **CSS**: Used for styling the web pages.
- **JavaScript**: Used for client-side scripting and handling form submissions.
- **DataTables**: jQuery plugin for enhancing HTML tables with advanced interaction controls.

### Prerequisites
- Rust and Cargo installed
- Docker installed
- Have a directory named 'db' in the root of the project with a password.txt inside containing the 
  password for the Postgres database.

### Running the Application
1. Start the Rust server:
    ```sh
    docker compose up --build
    ```
2. Open your web browser and navigate to `http://localhost:3000`

## File Structure
- `src/`: Contains the Rust source code
  - `src/models`: Contains the api models
  - `src/controllers`: Contains the api controllers
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

## Usage

- **Submit a Session**: Fill out the form on the topics page and click "Submit".
- **View Sessions**: The topics table will display all submitted topics with options to delete,
  search, sort, and paginate.
- **Create Empty Schedule**: Navigate to the schedules page and fill in and submit the rooms form.
- **Populate Schedule**: Click the "Populate Schedule" button to automatically assign topics to rooms.
- **Manually Adjust Schedule**: Drag and drop topics to adjust the schedule.

## Additional Notes
- Sometimes during the development process you may find the migrations in an unhealthy state. If this
  happens, you can reset the database by running the following commands:
    ```sh
    docker compose down
    docker volume rm thesis_db_data
    docker compose up --build
    ```

## License
This project is licensed under the MIT License.