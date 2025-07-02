use chrono::NaiveTime;
use clap::Parser;
use dotenvy::dotenv;
use fake::faker::internet::raw::*;
use fake::faker::name::raw::*;
use fake::locales::EN;
use fake::Fake;
use rand::Rng;
use serde_json::Value;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use unconfrs::{
    config::AppState,
    models::auth_model::Backend,
    models::auth_model::RegistrationRequest,
    models::room_model::{rooms_add, CreateRoomsForm, Room},
    models::timeslot_model::{timeslots_add, TimeslotForm, TimeslotRequest},
};

#[derive(Debug)]
enum CliError {
    Io(tokio::io::Error),
    Json(serde_json::Error),
    Args(String),
}

impl From<tokio::io::Error> for CliError {
    fn from(err: tokio::io::Error) -> Self {
        CliError::Io(err)
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Path to the JSON configuration file
    /// JSON file usecase is currently not implemented
    #[arg(conflicts_with_all = ["rooms", "timeslots", "users", "sessions"])]
    json_file: Option<PathBuf>,

    /// Number of rooms
    #[arg(long, default_value = "3")]
    rooms: Option<u32>,

    /// Number of timeslots
    #[arg(long, default_value = "5")]
    timeslots: Option<u32>,

    /// Number of users
    #[arg(long, default_value = "40")]
    users: Option<u32>,

    /// Number of created sessions
    #[arg(long, default_value = "20")]
    sessions: Option<u32>,
}

impl Cli {
    async fn validate(&self) -> Result<ValidatedParams, CliError> {
        if self.json_file.is_none() && [self.rooms, self.timeslots, self.users, self.sessions].iter().any(|&x| x.is_none()) {
            return Err(CliError::Args(String::from("Must provide either a JSON file or all parameters")));
        }

        if self.json_file.is_some() {
            let file_contents = tokio::fs::read_to_string(self.json_file.clone().unwrap()).await?;
            match serde_json::from_str(&file_contents) {
                Ok(val) => Ok(ValidatedParams::JsonConfig(val)),
                Err(err) => Err(CliError::Json(err)),
            }
        } else {
            Ok(ValidatedParams::PassedArgs(
                Params {
                    rooms: self.rooms.unwrap(),
                    timeslots: self.timeslots.unwrap(),
                    users: self.users.unwrap(),
                    sessions: self.sessions.unwrap(),
                }
            ))
        }
    }
}

struct Params {
    rooms: u32,
    timeslots: u32,
    users: u32,
    sessions: u32,
}

enum ValidatedParams {
    PassedArgs(Params),
    JsonConfig(Value),
}

#[tokio::main]
async fn main() {
    // load env vars
    dotenv().ok();

    let cli = Cli::parse();

    let validated_params = match cli.validate().await {
        Ok(params) => params,
        Err(CliError::Io(err)) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
        Err(CliError::Json(err)) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
        Err(CliError::Args(err)) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
    };

    match validated_params {
        ValidatedParams::JsonConfig(config) => {
            println!("Using JSON configuration from: {}", cli.json_file.clone().unwrap().display());
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
            unimplemented!();
        }
        ValidatedParams::PassedArgs(params) => {
            println!("Using parameters:");
            println!("Rooms: {}", params.rooms);
            println!("Timeslots: {}", params.timeslots);
            println!("Users: {}", params.users);
            println!("Sessions: {}", params.sessions);

            match params.generate_data().await {
                Ok(()) => println!("Successfully generated data"),
                Err(err) => {
                    eprintln!("Error: {err:?}");
                    std::process::exit(1);
                }
            }
        }
    }
}

impl Params {
    async fn generate_data(&self) -> Result<(), Box<dyn Error>> {
        let app_state = Arc::new(RwLock::new(AppState::new().await?));
        let app_state_lock = app_state.read().await;
        let db_pool = &app_state_lock.unconf_data.read().await.unconf_db;
        let backend = &app_state_lock.auth_backend;
        self.generate_users(backend).await?;
        self.generate_rooms(db_pool).await?;
        self.generate_timeslots(db_pool).await?;
        self.generate_sessions(db_pool).await?;
        self.generate_votes(db_pool).await?;

        Ok(())
    }

    async fn generate_users(&self, backend: &Backend) -> Result<(), Box<dyn Error>> {
        for _ in 1..=self.users {
            let user = RegistrationRequest::new(
                FirstName(EN).fake(),
                LastName(EN).fake(),
                SafeEmail(EN).fake(),
                String::from("password"),
            );
            backend.register(user).await?;
        }
        Ok(())
    }

    async fn generate_rooms(&self, db_pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        let mut rooms: Vec<Room> = vec![];
        for i in 1..=self.rooms {
            let room = Room::new(
                None,
                20,
                format!("Room {i}"),
                format!("Loc {i}"),
            );
            rooms.push(room);
        }
        let rooms_form = CreateRoomsForm { rooms };
        match rooms_add(db_pool, rooms_form).await {
            Ok(_) => Ok(()),
            Err(err) => {
                dbg!(&err);
                Err(err)
            }
        }
    }

    async fn generate_timeslots(&self, db_pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        let mut start_time = NaiveTime::parse_from_str("08:00", "%H:%M")?;
        let mut end_time = NaiveTime::parse_from_str("08:30", "%H:%M")?;
        let duration = end_time - start_time;
        let mut timeslots = vec![];
        dbg!(duration);
        for _ in 1..=self.timeslots {
            let timeslot = TimeslotForm {
                start_time: start_time.format("%H:%M").to_string(),
                duration: duration.num_minutes() as i32,
                assignments: vec![],
            };

            timeslots.push(timeslot);
            start_time += duration;
            end_time += duration;
        }

        let timeslots_req = TimeslotRequest {
            timeslots,
        };

        timeslots_add(db_pool, timeslots_req).await?;

        Ok(())
    }

    async fn generate_sessions(&self, db_pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        let user_ids = sqlx::query_scalar::<Postgres, i32>("SELECT id FROM users")
            .fetch_all(db_pool)
            .await?;

        for _ in 1..=self.sessions {
            let mut rng = rand::rng();
            let random_index = rng.random_range(0..user_ids.len());
            let user_id = user_ids[random_index];
            let title = (3..10).fake::<String>();
            let content = (8..15).fake::<String>();
            let votes = 0;

            sqlx::query!(
                "INSERT INTO sessions (user_id, title, content, votes) VALUES ($1, $2, $3, $4)",
                user_id,
                title,
                content,
                votes,
            )
                .execute(db_pool)
                .await?;
        }
        Ok(())
    }

    async fn generate_votes(&self, db_pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        let user_ids = sqlx::query_scalar!("SELECT id FROM users")
            .fetch_all(db_pool)
            .await?;

        let session_ids = sqlx::query_scalar!("SELECT id FROM sessions")
            .fetch_all(db_pool)
            .await?;

        let mut rng = rand::rng();
        for user_id in user_ids {
            let user_does_voting = rng.random_bool(9.0 / 10.0);
            if !user_does_voting {
                continue;
            }

            let number_of_votes = rng.random_range(2..=6);
            let mut voted_on_sessions: Vec<i32> = vec![];
            for _ in 0..number_of_votes {
                let mut session_index = rng.random_range(0..session_ids.len());
                let mut session_id = session_ids[session_index];
                while voted_on_sessions.contains(&session_id) {
                    session_index = rng.random_range(0..session_ids.len());
                    session_id = session_ids[session_index];
                }

                sqlx::query!(
                    "INSERT INTO user_votes (user_id, session_id) VALUES ($1, $2)",
                    user_id,
                    session_id,
                )
                    .execute(db_pool)
                    .await?;

                sqlx::query!(
                    "UPDATE sessions SET votes = votes + 1 WHERE id = ($1)",
                    session_id,
                )
                    .execute(db_pool)
                    .await?;

                voted_on_sessions.push(session_id);
            }
        }

        Ok(())
    }
}
