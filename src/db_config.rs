use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::error::Error;
use tracing::trace;

pub async fn db_setup() -> Result<Pool<Postgres>, Box<dyn Error>> {
    use std::env::var;
    use std::fs;

    let pg_user = var("PG_USER")?;
    println!("user");
    tracing::info!("pg user");
    let password_file = var("PG_PASSWORDFILE")?;
    tracing::info!("pw file");
    let password = fs::read_to_string(password_file)?;
    let pg_host = var("PG_HOST")?;
    tracing::info!("pg host");
    let pg_dbname = var("PG_DBNAME")?;
    tracing::info!("pg dbname");

    let connection = db_connect(&pg_user, &password, &pg_host, &pg_dbname).await?;
    tracing::info!("Connected to: {:?}", connection);
    tracing::info!("Running migrations if any are needed");
    sqlx::migrate!().run(&connection).await?;

    Ok(connection)
}

async fn db_connect(
    pg_user: &str,
    password: &str,
    pg_host: &str,
    pg_dbname: &str,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let url = format!(
        "postgresql://{}:{}@{}:5432/{}",
        pg_user,
        password.trim(),
        pg_host,
        pg_dbname,
    );

    trace!("Attempting Connection to: {}", &url);

    match PgPoolOptions::new().connect(&url).await {
        Ok(connection) => Ok(connection),
        Err(e) => Err(e),
    }
}
