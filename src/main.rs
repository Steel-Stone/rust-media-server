use constants::server_constants;
use env_logger::Env;
use log::info;

pub mod api;
pub mod application;
pub mod clients;
pub mod constants;
pub mod service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    info!("Launching app!");
    application::start().await
}
