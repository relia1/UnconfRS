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
        let jwt_env = std::env::var("JWT_SECRET");
        if jwt_env.as_ref().is_ok_and(|token| !token.is_empty()) {
            Ok(Self {
                unconf_data: Arc::new(RwLock::new(UnconfData::new().await?)),
                jwt_secret: Arc::new(RwLock::new(jwt_env.unwrap())),
            })
        } else {
            Err("JWT_SECRET not set. Set the JWT_SECRET field in the compose.yaml".into())
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
