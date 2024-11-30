use crate::db_config::*;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{RwLock};

pub  struct AppState {
    pub unconf_data: Arc<RwLock<UnconfData>>,
    pub jwt_secret: Arc<RwLock<String>>,
}

impl AppState {
    pub  async fn new() -> Result<Self, Box<dyn Error>> {
        let get_secret = || -> Result<String, Box<dyn Error>> {
            let secret_file = std::env::var("JWT_SECRETFILE")?;
            let secret = std::fs::read_to_string(secret_file)?.trim().to_owned();
            Ok(secret)
        };
        if let Ok(jwt_secret) = get_secret() {
            Ok(Self {
                unconf_data: Arc::new(RwLock::new(UnconfData::new().await?)),
                jwt_secret: Arc::new(RwLock::new(jwt_secret)),
            })
        } else {
            Err("JWT_SECRET not set up properly. See the README".into())
        }
    }
}
/// A question bank that stores and manages questions and their answers
#[derive(Debug)]
pub struct UnconfData {
    pub unconf_db: Pool<Postgres>,
}

impl UnconfData {
    /// Creates a new `UnconfData` instance.
    ///
    /// # Parameters
    ///
    /// * `db_path`: The path to the file that will store the questions.
    ///
    /// # Returns
    ///
    /// A new `UnconfData` instance, or an error if the database cannot be initialized
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            unconf_db: db_setup().await?,
        })
    }
}
