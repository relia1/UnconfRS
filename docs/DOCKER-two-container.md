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
