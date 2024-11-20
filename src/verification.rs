use sqlx::PgPool;
use tracing::instrument;

#[instrument(name = "Verify Account Credentials")]
pub async fn verify_account_credentials(
    public_key: &str,
    pool: &PgPool,
) -> Result<Option<i32>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT token_amount FROM accounts WHERE public_key = $1",
        public_key
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to verify account credentials: {:?}", e);
        e
    })?;

    Ok(result.map(|record| record.token_amount))
}

#[instrument(name = "Add New Account")]
pub async fn add_account(public_key: &str, pool: &PgPool) -> Result<i32, sqlx::Error> {
    let initial_token_amount = 0;
    sqlx::query!(
        "INSERT INTO accounts (public_key, token_amount) VALUES ($1, $2)",
        public_key,
        initial_token_amount
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add account: {:?}", e);
        e
    })?;

    Ok(initial_token_amount)
}
