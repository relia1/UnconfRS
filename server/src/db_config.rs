use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::error::Error;
extern crate thiserror;
use tracing::info;

#[derive(Debug, thiserror::Error)]
enum DbSetupError {
     #[error("bad password variable: expected POSTGRES_PASSWORD, got {0}")]
     BadPwVariable(String),
     #[error("no password variable: expected POSTGRES_PASSWORD")]
     NoPwVariable,
}

/// Sets up the database connection pool
///
/// This function reads the environment variables for the database connection and sets up the
/// connection pool, then runs any migrations that are needed.
///
/// # Returns
/// `Ok(Pool<Postgres>)` if the connection is successful, or an error if not.
///
/// # Errors
/// This function will return an error if:
/// - The environment variables are not set
/// - The password file cannot be read
/// - The connection to the database cannot be established
/// - The migrations cannot be run
pub async fn db_setup() -> Result<Pool<Postgres>, Box<dyn Error>> {
    use std::env::var;
    use std::fs;

    let pg_user = var("PG_USER")?;
    let password_file = var("PG_PASSWORDFILE")?;
    let password_var = fs::read_to_string(password_file)?;
    let password_expr = password_var.trim().split_once('=');
    let password = if let Some((password_var, password)) = password_expr {
        if password_var != "POSTGRES_PASSWORD" {
            return DbSetupError::BadPwVariable(password_var.to_owned()).into();
        }
        password
    } else {
        return DbSetupError::NoPwVariable.into();
    };
    let pg_host = var("PG_HOST")?;
    let pg_port = var("PG_PORT")?;
    let pg_dbname = var("PG_DBNAME")?;

    let connection = db_connect(&pg_user, password, &pg_host, &pg_port, &pg_dbname).await?;
    info!("Connected to: {:?}", connection);
    info!("Running migrations if any are needed");
    sqlx::migrate!().run(&connection).await?;

    Ok(connection)
}

/// Connects to the database
///
/// This function connects to the database using the provided configuration.
///
/// # Parameters
/// - `pg_user`: The username for the database
/// - `password`: The password for the database
/// - `pg_host`: The hostname for the database
/// - `pg_dbname`: The name of the database
///
/// # Returns
/// `Ok(Pool<Postgres>)` if the connection is successful, or an error if not.
///
/// # Errors
/// This function will return an error if the connection to the database cannot be established.
async fn db_connect(
    pg_user: &str,
    password: &str,
    pg_host: &str,
    pg_port: &str,
    pg_dbname: &str,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        pg_user,
        password.trim(),
        pg_host,
        pg_port,
        pg_dbname,
    );

    let url_without_pw = format!(
        "postgresql://{pg_user}:REDACTED@{pg_host}:{pg_port}/{pg_dbname}",
    );

    println!("Connecting to: {url_without_pw}");

    info!("Attempting Connection to: {}", url_without_pw);

    match PgPoolOptions::new().connect(&url).await {
        Ok(connection) => Ok(connection),
        Err(e) => Err(e),
    }
}
