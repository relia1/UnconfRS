use crate::db_config::*;
use crate::models::auth_model::Backend;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The application state
///
/// This struct holds the application state and JWT secret both wrapped in an Arc and RwLock
///
/// # Fields
/// - `unconf_data`: Thread-safe storage for the application data
/// - `jwt_secret`: Thread-safe storage for the JWT secret
pub struct AppState {
    pub unconf_data: Arc<RwLock<UnconfData>>,
    pub auth_backend: Backend,
}

impl AppState {
    /// Creates a new `AppState` instance.
    ///
    /// # Returns
    /// `Ok(AppState)`, or an error if unable to initialize UnconfData
    ///
    /// # Errors
    /// This function will return an error if:
    /// - UnconfData cannot be initialized
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let unconf_data = UnconfData::new().await?;
        let db_pool = unconf_data.unconf_db.clone();
        let auth_backend = Backend::new(db_pool);

        Ok(Self {
            unconf_data: Arc::new(RwLock::new(UnconfData::new().await?)),
            auth_backend,
        })
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
