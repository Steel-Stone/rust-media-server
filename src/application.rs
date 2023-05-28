use crate::api::routes;
use crate::server_constants;
use crate::service;
use actix_web::{App, HttpServer};
use log::info;

pub async fn start() -> std::io::Result<()> {
    service::webservice::init().await;
    info!("Starting web api");
    HttpServer::new(|| App::new().service(routes::hello::world))
        .bind((server_constants::SERVER_IP, server_constants::SERVER_PORT))?
        .run()
        .await
}
