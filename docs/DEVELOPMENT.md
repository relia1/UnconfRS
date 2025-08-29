# Development

[<- Back to Main README](../README.md) | [Setup Guide](SETUP.md)

## Project Structure

* Frontend is in Javascript in `web/`

* Backend is in Rust in `server/`

* Scheduler library is in Rust in `scheduler/`

* Some scheduler testing is in Rust in `test_unconf/`

  `src/bin/test_unconf.rs`: A testing utility for generating complete unconferences in order to test the scheduling
  aspects of the application. Running `cargo run --bin test_unconf --release` without any additional arguments will
  generate an unconference with these qualities

  - 3 rooms
    - Room name and location will be of the format `Room {1..=num_of_rooms}` and `Loc {1..=num_of_rooms}`
  - 5 timeslots
    - Is set to start at 08:00 with a duration of 30 minutes, each additional timeslot is offset 30 minutes from the start
      of the previous up to the number of timeslots
  - 40 users
    - Each user will be generated with a random first name, last name, email, but have a hard-coded password of `password`
  - 20 sessions
    - Each session will belong to one of the users randomly, and it'll have random text for its title and content
  - Voting Distribution
    - Makes it so each user has a 90% likelyhood of voting for any session
    - If a user is a voting one they'll vote on between `2..=6` sessions (no duplicate voting on sessions for a user)

## Development Workflow

### Setting Up

1. Follow the [Setup](SETUP.md) to get the application running
2. For active development, use the hybrid approach (Docker for the DB and running the application local):
    ```sh
   docker compose up -d db
   cargo run --bin unconfrs
    ```

### Making Changes

#### Adding a New Endpoint

- Define the route in the appropriate routes file `src/routes/`
- Implement the handler in `src/controllers`
- Implement the new model in `src/models`

#### Adding a New Page

- Create HTML template in `web/templates`
- Add route in `src/routes/site_routes.rs`
- Add handler in `src/controllers/site_handler.rs`
- Add any JS in `web/scripts`
- Add any CSS in `web/styles`

#### Making Database Changes

- To keep .sqlx/ up-to-date run cargo sqlx prepare before any commit adding/modifying any querries

---

**Related Documentation:**

- [Architecture](ARCHITECTURE.md) - Technologies and system design
- [User Guide](USER_GUIDE.md) - Learn how to use the application
