use actix_web::{post, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;
use tracing::error;
use tracing::instrument;

// Import the functions from the user_verification module
use crate::verification::{add_user, verify_user_credentials};

#[derive(Debug, Deserialize)]
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

#[instrument(name = "GENERATE")]
#[post("/generate")]
pub async fn generate(form: web::Json<GenerateRequest>, pool: web::Data<PgPool>) -> impl Responder {
    if !form.is_connected || form.public_key.is_none() {
        return HttpResponse::BadRequest().json(GenerateResponse {
            success: false,
            message: "User is not connected or public key is missing".into(),
            image: None,
            token_amount: None,
        });
    }

    let public_key = form.public_key.as_ref().unwrap();

    match verify_user_credentials(public_key, pool.get_ref()).await {
        Ok(Some(token_amount)) => {
            // User exists, proceed with image generation
            // let image_path = generate_image(form.gen_params);
            let image_path = "./static/chimp.png"; // Placeholder path
                                                   // i
            match fs::read(image_path) {
                Ok(image_data) => {
                    let encoded_image = general_purpose::STANDARD.encode(image_data);
                    HttpResponse::Ok().json(GenerateResponse {
                        success: true,
                        message: "Image generated successfully".into(),
                        image: Some(encoded_image),
                        token_amount: Some(token_amount),
                    })
                }
                Err(e) => {
                    error!("Failed to read image file: {:?}", e);
                    HttpResponse::InternalServerError().json(GenerateResponse {
                        success: false,
                        message: "Failed to read image file".into(),
                        image: None,
                        token_amount: Some(token_amount),
                    })
                }
            }
        }
        Ok(None) => {
            // User does not exist, attempt to add them
            match add_user(public_key, pool.get_ref()).await {
                Ok(token_amount) => HttpResponse::Created().json(GenerateResponse {
                    success: true,
                    message: "You currently have no generation tokens. Add tokens to your balance to generate".into(),
                    image: None,
                    token_amount: Some(token_amount),
                }),
                Err(_) => HttpResponse::InternalServerError().json(GenerateResponse {
                    success: false,
                    message: "Failed to add user to the database".into(),
                    image: None,
                    token_amount: None,
                }),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(GenerateResponse {
            success: false,
            message: format!("Failed to verify user credentials: {e}"),
            image: None,
            token_amount: None,
        }),
    }
}
