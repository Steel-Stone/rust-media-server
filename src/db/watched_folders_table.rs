use std::path::Path;
use std::sync::Arc;
use log::info;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Error, Connection};
use strum_macros::Display;
use uuid::Uuid;
use crate::db::databasehandler::ConnectionProvider;

pub struct WatchedFoldersDb {
    pool: Arc<dyn ConnectionProvider>,
}

pub struct WatchedFolder {
    pub path: String,
    pub folder_name: String
}

impl WatchedFoldersDb {
    pub fn new(connection_provider: Arc<dyn ConnectionProvider>) -> WatchedFoldersDb {
        WatchedFoldersDb { pool: connection_provider }
    }

    pub fn create(&self, path: &String) -> Result<WatchedFolder, Error> {
        let p = Path::new(path);
        let folder_name = p.file_name().unwrap();
        info!("foldername: {}", folder_name.to_owned().into_string().unwrap());
        match self
            .pool
            .get_connection()
            .execute(
                "INSERT INTO WatchedFolders (path, folder_name) VALUES (?1, ?2)"
                , &[path, &folder_name.to_owned().into_string().unwrap()])
        {
            Ok(_) => self.get(path),
            Err(error) => Err(error),
        }
    }

    pub fn get(&self, path: &String) -> Result<WatchedFolder, Error> {
        let connection = self.pool.get_connection();
        let mut stmt = connection
            .prepare("SELECT path, folder_name FROM WatchedFolders WHERE path = ?1")?;

        stmt.query_row([path], |row| Ok(WatchedFolder { path: row.get(0)?, folder_name: row.get(1)? }))
    }

    pub fn list(&self) -> Result<Vec<WatchedFolder>, Error> {
        let connection = self.pool.get_connection();
        let mut stmt = connection.prepare("SELECT path, folder_name FROM WatchedFolders")?;

        let x = stmt
            .query_map([], |row| Ok(WatchedFolder { path: row.get(0)?, folder_name: row.get(1)? }))?
            .collect();
        x
    }

    pub fn delete(&self, path: &String) -> Result<usize, Error> {
        self.pool
            .get_connection()
            .execute("DELETE FROM WatchedFolders WHERE path = ?1", &[path])
    }
}
