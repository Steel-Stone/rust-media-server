use crate::service::video;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
async fn world() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/stream/playlist.m3u8")]
async fn movie() -> impl Responder {
    let movie_playlist = video::movies::get_movie().unwrap();
    HttpResponse::Ok()
        .insert_header(("application", "x-mpegURL"))
        .insert_header(("Accept-Ranges", "bytes"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .body(movie_playlist)
}

#[get("/stream/{filename}")]
async fn play_movie<'r>(path: web::Path<String>) -> impl Responder {
    println!("play_movie called");
    let file_name = path.into_inner();
    let movie_chunk = video::movies::play_movie(file_name).unwrap();

    HttpResponse::Ok()
        .insert_header(("application", "x-mpegURL"))
        .insert_header(("Accept-Ranges", "bytes"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .body(movie_chunk)
}
