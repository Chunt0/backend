use actix_web::{get, HttpResponse};
use tracing::instrument;

#[instrument(name = "HEALTH_CHECK")]
#[get("/health_check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
