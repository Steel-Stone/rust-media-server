use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn world() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
