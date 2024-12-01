use crate::db_config::*;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{RwLock};

/// The application state
/// 
/// This struct holds the application state and JWT secret both wrapped in an Arc and RwLock
/// 
/// # Fields
/// - `unconf_data`: Thread-safe storage for the application data
/// - `jwt_secret`: Thread-safe storage for the JWT secret
pub  struct AppState {
    pub unconf_data: Arc<RwLock<UnconfData>>,
    pub jwt_secret: Arc<RwLock<String>>,
}

impl AppState {
    /// Creates a new `AppState` instance.
    /// 
    /// # Environment Variables
    /// - `JWT_SECRETFILE`: The path to the file that contains the JWT secret
    /// 
    /// # Returns
    /// `Ok(AppState)` if the JWT secret is set up properly, or an error if not.
    /// 
    /// # Errors
    /// This function will return an error if:
    /// - `JWT_SECRETFILE` is not set
    /// - The file specified by `JWT_SECRETFILE` does not exist
    /// - UnconfData cannot be initialized
    pub  async fn new() -> Result<Self, Box<dyn Error>> {
        let get_secret = || -> Result<String, Box<dyn Error>> {
            let secret_file = std::env::var("JWT_SECRETFILE")?;
            let secret = std::fs::read_to_string(secret_file)?.trim().to_owned();
            Ok(secret)
        };
        match get_secret() {
            Ok(jwt_secret) => {
                Ok(Self {
                    unconf_data: Arc::new(RwLock::new(UnconfData::new().await?)),
                    jwt_secret: Arc::new(RwLock::new(jwt_secret)),
                })
            }
            Err(e) => {
                Err(format!("JWT_SECRET not set up properly. See the README\n({})", e).into())
            }
        }
    }
}

/// The struct holds the database connection pool
/// 
/// # Fields
/// - `unconf_db`: The database connection pool
#[derive(Debug)]
pub struct UnconfData {
    pub unconf_db: Pool<Postgres>,
}

impl UnconfData {
    /// Creates a new `UnconfData` instance.
    ///
    /// This function initializes the database connection pool using the `db_setup` function.
    ///
    /// # Returns
    /// `Ok(UnconfData)` if the database connection pool is set up properly, or an error if not.
    /// 
    /// # Errors
    /// This function will return an error if the database connection pool cannot be initialized.
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            unconf_db: db_setup().await?,
        })
    }
}
