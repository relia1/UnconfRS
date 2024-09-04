use crate::db_config::*;
use sqlx::{Pool, Postgres};
use std::error::Error;

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
