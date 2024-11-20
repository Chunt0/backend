use sqlx::PgPool;
use tracing::instrument;

#[instrument(name = "Verify User Credentials")]
pub async fn verify_user_credentials(
    public_key: &str,
    pool: &PgPool,
) -> Result<Option<i32>, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT token_amount FROM accounts WHERE public_key = $1",
        public_key
    )
    .fetch_optional(pool)   sqlx migrate run


---------------------------User---------------------------


    .await?;

    Ok(result.map(|record| record.token_amount))
}

#[instrument(name = "Add New User")]
pub async fn add_user(public_key: &str, pool: &PgPool) -> Result<i32, sqlx::Error> {
    let initial_token_amount = 0;
    sqlx::query!(
        "INSERT INTO accounts (public_key, token_amount) VALUES ($1, $2)",
        public_key,
        initial_token_amount
    )
    .execute(pool)
    .await?;

    Ok(initial_token_amount)
}
