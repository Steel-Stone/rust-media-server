use crate::api::routes;
use crate::db::databasehandler::DatabaseHandler;
use crate::db::watched_folders_table::WatchedFoldersDb;
use crate::folder_watcher::watcher;
use crate::gui::page;
use crate::gui::page::Flags;
use crate::server_constants;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use log::info;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use std::thread;

pub async fn start() -> std::io::Result<()> {
    // service::webservice::init().await;

    let db_handler = DatabaseHandler::new(SqliteConnectionManager::file("file.db"));
    //Pooled connection probably should not be in the db_handler if we just get the connection from it
    //Maybe the WatchedFoldersDb should take a db handler
    let pooled_connection = db_handler.get_connection();
    let conn: &'static r2d2::PooledConnection<SqliteConnectionManager> = &pooled_connection;
    let watched_folders_db = WatchedFoldersDb::new(&conn);

    let _ = watched_folders_db
        .create(&"./videos".to_owned())
        .map(|succ| info!("{}", succ.path))
        .map_err(|err| info!("Problem parsing arguments: {err}"));

    let mut watcher: watcher::FolderWatcher = watcher::FolderWatcher::new().unwrap();

    thread::spawn(move ||{
        info!("Starting folder watcher.");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {watcher.async_watch(Path::new("./videos")).await.unwrap()});
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
    page::start(Flags { watched_folders_db }).unwrap();

    server_handler.stop(false).await;

    std::io::Result::Ok(())
}
