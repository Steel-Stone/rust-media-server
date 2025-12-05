use std::path::Path;
use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Error, Connection};
use strum_macros::Display;
use uuid::Uuid;
use crate::db::databasehandler::ConnectionProvider;

pub struct VideosDb {
    pool: Arc<dyn ConnectionProvider>,
}

pub struct Video {
    pub id: String,
    pub file_name: String,
    pub watched_folder: String
}

impl VideosDb {
    pub fn new(connection_provider: Arc<dyn ConnectionProvider>) -> VideosDb {
        VideosDb { pool: connection_provider }
    }

    pub fn create(&self, file_name: &String, watched_folder: &String) -> Result<Video, Error> {
        let id = Uuid::new_v4();
        match self
            .pool
            .get_connection()
            .execute(
                "INSERT INTO Videos (id, file_name, watched_folder) VALUES (?1, ?2, ?3)"
                , &[&id.to_string() , file_name, watched_folder])
        {
            Ok(_) => self.get(&id.to_string()),
            Err(error) => Err(error),
        }
    }

    pub fn get(&self, id: &String) -> Result<Video, Error> {
        let connection = self.pool.get_connection();
        let mut stmt = connection
            .prepare("SELECT id, file_name, watched_folder FROM Videos WHERE id = ?1")?;

        stmt.query_row([id], |row| Ok(Video { id: row.get(0)?, file_name: row.get(1)?,  watched_folder: row.get(2)? }))
    }

    pub fn list(&self) -> Result<Vec<Video>, Error> {
        let connection = self.pool.get_connection();
        let mut stmt = connection.prepare("SELECT path, id, folder_name FROM Videos")?;

        let x = stmt
            .query_map([], |row| Ok(Video { id: row.get(0)?, file_name: row.get(1)?,  watched_folder: row.get(2)? }))?
            .collect();
        x
    }

    pub fn delete(&self, id: &String) -> Result<usize, Error> {
        self.pool
            .get_connection()
            .execute("DELETE FROM Videos WHERE id = ?1", &[id])
    }
}
