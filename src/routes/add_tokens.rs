use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{Error, PgPool};
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct AddTokensRequest {
    is_connected: bool,
    public_key: Option<String>,
    token_addition: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AddTokensResponse {
    success: bool,
    message: String,
    token_amount: Option<i32>,
}

async fn update_token_amount(
    public_key: &str,
    token_addition: i32,
    pool: &PgPool,
) -> Result<Option<i32>, sqlx::Error> {
    let original_amount = sqlx::query!(
        "SELECT token_amount FROM accounts WHERE public_key = $1",
        public_key
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to retrieve users current token amount: {:?}", e);
        e
    })?;

    let token_amount = match original_amount {
        Some(record) => record.token_amount + token_addition,
        None => {
            tracing::error!("No account found or token amount does not exist");
            return Err(Error::RowNotFound);
        }
    };

    sqlx::query!(
        "UPDATE accounts SET token_amount = $1 WHERE public_key = $2",
        token_amount,
        public_key
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update token_amount: {:?}", e);
        e
    })?;

    let result = sqlx::query!(
        "SELECT token_amount FROM accounts WHERE public_key = $1",
        public_key
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to read token amount: {:?}", e);
        e
    })?;
    Ok(result.map(|record| record.token_amount))
}

#[instrument(name = "ADD TOKENS")]
#[post("/add_tokens")]
pub async fn add_tokens(
    form: web::Json<AddTokensRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    if !form.is_connected || form.public_key.is_none() || form.token_addition.is_none() {
        return HttpResponse::BadRequest().json(AddTokensResponse {
            success: false,
            message: "User is not connected or public key is missing".into(),
            token_amount: None,
        });
    }

    let public_key = form.public_key.as_ref().unwrap();
    let token_addition = form.token_addition.unwrap();
    match update_token_amount(public_key, token_addition, &pool).await {
        Ok(token_amount) => HttpResponse::Ok().json(AddTokensResponse {
            success: true,
            message: format!("You now have {:#?}", token_amount),
            token_amount,
        }),
        Err(e) => {
            tracing::error!("Failed to update token amount: {:?}", e);
            HttpResponse::InternalServerError().json(AddTokensResponse {
                success: false,
                message: "Failed to update token amount".into(),
                token_amount: None,
            })
        }
    }
}
