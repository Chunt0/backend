use actix_web::{post, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use std::process::Command;
use tempfile::NamedTempFile;
use tracing::instrument;

use crate::verification::{add_account, verify_account_credentials};

#[derive(Debug, Deserialize, Clone)]
pub struct GenerationParams {
    pos_prompt: String,
    neg_prompt: String,
    prompt_strength: f32,
    batch_size: u32,
    size: (u32, u32),
    loras: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    is_connected: bool,
    public_key: Option<String>,
    gen_params: GenerationParams,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    success: bool,
    message: String,
    image: Option<String>,
    token_amount: Option<i32>,
}

impl GenerateResponse {
    fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            image: None,
            token_amount: None,
        }
    }

    fn success(image: String, token_amount: i32) -> Self {
        Self {
            success: true,
            message: "Image generated".into(),
            image: Some(image),
            token_amount: Some(token_amount),
        }
    }

    fn insufficient_tokens(image: String) -> Self {
        Self {
            success: true,
            message: "Insufficient token balance".into(),
            image: Some(image),
            token_amount: Some(0),
        }
    }

    fn new_account(image: String, token_amount: i32) -> Self {
        Self {
            success: true,
            message:
                "You currently have no generation tokens. Add tokens to your balance to generate"
                    .into(),
            image: Some(image),
            token_amount: Some(token_amount),
        }
    }
}

pub fn invalid_public_key(public_key: &Option<String>) -> bool {
    let unsupported_chars: HashSet<char> = "OIl0".chars().collect();

    if let Some(key) = public_key {
        // Check if the key is of the correct length (e.g., 44 characters for base58 encoded keys)
        if key.len() < 32 || key.len() > 44 {
            return true;
        }

        // Check if the key contains only valid characters
        for ch in key.chars() {
            if !ch.is_ascii_alphanumeric() || unsupported_chars.contains(&ch) {
                return true;
            }
        }

        // If all checks pass, the key is valid
        false
    } else {
        // If public_key is None, it's not valid
        true
    }
}

fn generate_image(gen_params: &GenerationParams) -> Result<String, Box<dyn std::error::Error>> {
    // Create a temporary file to store the generated image
    let temp_file = NamedTempFile::new()?;

    // Prepare the command to run the Python script
    let output = Command::new("python3")
        .arg("./scripts/python/generate.py")
        .arg("--pos_prompt")
        .arg(&gen_params.pos_prompt)
        .arg("--neg_prompt")
        .arg(&gen_params.neg_prompt)
        .arg("--prompt_strength")
        .arg(gen_params.prompt_strength.to_string())
        .arg("--batch_size")
        .arg(gen_params.batch_size.to_string())
        .arg("--size")
        .arg(format!("{}x{}", gen_params.size.0, gen_params.size.1))
        .arg("--loras")
        .arg(gen_params.loras.join(","))
        .arg("--output")
        .arg(temp_file.path())
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script failed: {}", error_message).into());
    }
    // Read the generated image

    let image_data = std::fs::read("./scripts/python/output/temp.png")?;

    // Encode the image data to base64
    Ok(general_purpose::STANDARD.encode(image_data))
}

async fn update_token_amount(
    token_amount: i32,
    public_key: &str,
    pool: &PgPool,
) -> Result<i32, sqlx::Error> {
    let updated_token_amount = token_amount - 1;
    sqlx::query!(
        "UPDATE accounts SET token_amount = $1 WHERE public_key = $2",
        updated_token_amount,
        public_key
    )
    .execute(pool)
    .await?;
    Ok(updated_token_amount)
}

async fn process_generation(
    public_key: &str,
    gen_params: &GenerationParams,
    pool: &PgPool,
) -> Result<GenerateResponse, Box<dyn std::error::Error>> {
    match verify_account_credentials(public_key, pool).await? {
        Some(token_amount) => {
            if token_amount == 0 {
                let chimp_image_data = std::fs::read("./static/chimp.png")?;
                let chimp_image: String = general_purpose::STANDARD.encode(chimp_image_data);
                return Ok(GenerateResponse::insufficient_tokens(chimp_image));
            }

            let generated_image = generate_image(gen_params)?;
            let updated_token_amount = update_token_amount(token_amount, public_key, pool).await?;

            Ok(GenerateResponse::success(
                generated_image,
                updated_token_amount,
            ))
        }
        None => {
            let chimp_image_data = std::fs::read("./static/chimp.png")?;
            let chimp_image: String = general_purpose::STANDARD.encode(chimp_image_data);
            let token_amount = add_account(public_key, pool).await?;
            Ok(GenerateResponse::new_account(chimp_image, token_amount))
        }
    }
}

#[instrument(name = "GENERATE", skip(pool))]
#[post("/generate")]
pub async fn generate(form: web::Json<GenerateRequest>, pool: web::Data<PgPool>) -> impl Responder {
    if !form.is_connected || invalid_public_key(&form.public_key) {
        return HttpResponse::BadRequest().json(GenerateResponse::error(
            "User is not connected or public key is missing".into(),
        ));
    }

    let public_key = form.public_key.as_ref().unwrap();
    match process_generation(public_key, &form.gen_params, pool.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Generation process failed: {:?}", e);
            HttpResponse::InternalServerError().json(GenerateResponse::error(
                "An error occurred during the generation process".into(),
            ))
        }
    }
}
