use actix_web::web::Form;
use actix_web::{HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
pub async fn subscribe(form: Form<FormData>) -> impl Responder {
    HttpResponse::Ok()
}
