use crate::api::routes;
use crate::db::databasehandler::{ConnectionProvider, DatabaseHandler};
use crate::db::videos_table::{self, VideosDb};
use crate::db::watched_folders_table::WatchedFoldersDb;
use crate::folder_watcher::watcher;
use crate::gui::page;
use crate::gui::page::Flags;
use crate::server_constants;
use crate::service;
use actix_cors::Cors;

use actix_web::rt;
use actix_web::{App, HttpServer};
use anyhow::Ok;
use iced::futures::TryFutureExt;
use log::info;
use r2d2_sqlite::SqliteConnectionManager;
use std::any;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tokio::task;

pub async fn start() -> std::io::Result<()> {
    // service::webservice::init().await;

    let db_handler = Arc::new(DatabaseHandler::new(SqliteConnectionManager::file(
        "file.db",
    )));
    //Pooled connection probably should not be in the db_handler if we just get the connection from it
    //Maybe the WatchedFoldersDb should take a db handler
    let conn: r2d2::PooledConnection<SqliteConnectionManager> = db_handler.get_connection();
    let watched_folders_db = WatchedFoldersDb::new(db_handler.clone());
    let videos_table_db = VideosDb::new(db_handler.clone());
    let created_watched_folder = watched_folders_db
        .create(&"./videos".to_owned())
        .map(|succ| info!("{}", succ.path))
        .map_err(|err| info!("Problem parsing arguments: {err}"));

    let mut watcher: watcher::FolderWatcher = watcher::FolderWatcher::new().unwrap();

    let folder_watcher_notifier = watcher.create_folders_to_watch_event_receiver();

    let c = watched_folders_db.list();

    let n = c.unwrap();

    let v = n.into_iter();

    let b = v
        .map(|watched_folder| PathBuf::from(watched_folder.path))
        .collect();

    thread::spawn(move || {
        info!("Starting folder watcher.");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async { watcher.async_watch(b).await.unwrap() });
    });

    let server = HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin();
        App::new()
            .wrap(cors)
            .service(routes::hello::world)
            .service(routes::hello::movie)
            .service(routes::hello::play_movie)
    })
    .workers(4)
    .bind((server_constants::SERVER_IP, server_constants::SERVER_PORT))
    .unwrap()
    .run();

    let server_handler = server.handle();

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        info!(
            "Starting web api at {}:{}.",
            server_constants::SERVER_IP,
            server_constants::SERVER_PORT
        );
        runtime.block_on(async { server.await })
    });

    info!("Starting ui.");
    page::start(Flags { watched_folders_db, folder_watcher_notifier }).unwrap();

    server_handler.stop(false).await;

    std::io::Result::Ok(())
}
