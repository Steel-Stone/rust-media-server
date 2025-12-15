use std::sync::Arc;
use rusqlite::{Error, Connection};

use crate::db::databasehandler::ConnectionProvider;

pub struct GlobalSettingsDb {
    pool: Arc<dyn ConnectionProvider>,
}

pub struct GlobalSettings {
    pub hls_videos_location: String,
}

impl GlobalSettingsDb {
    pub fn new(connection_provider: Arc<dyn ConnectionProvider>) -> GlobalSettingsDb {
        // TODO Try to create an Options as it should always exist
        GlobalSettingsDb { pool: connection_provider }
    }

    pub fn create(&self, hls_videos_location: &String) -> Result<GlobalSettings, Error> {
        match self
            .pool
            .get_connection()
            .execute(
                "INSERT INTO GlobalSettings (hls_videos_location) VALUES (?1)"
                , &[hls_videos_location])
        {
            Ok(_) => self.get(),
            Err(error) => Err(error),
        }
    }

    pub fn get(&self) -> Result<GlobalSettings, Error> {
        let connection = self.pool.get_connection();
        let mut stmt = connection
            .prepare("SELECT hls_videos_location FROM GlobalSettings")?;

        stmt.query_row([], |row| Ok(GlobalSettings { hls_videos_location: row.get(0)? }))
    }
}