use std::{format, io::Cursor, path::PathBuf, println, sync::Arc};

use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use base64::prelude::*;
use hmac::{Hmac, KeyInit, Mac};
use image::imageops::FilterType;
use sha1::Sha1;
use std::env;

type HmacSha1 = Hmac<Sha1>;

struct AppConfig {
    secret_key: String,
    base_fs_path: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let secret_key =
        env::var("SECRET_KEY").expect("The SECRET_KEY environment variable must be set.");
    let base_fs_path =
        env::var("BASE_FS_PATH").expect("The BASE_FS_PATH environment variable must be set.");

    let config = Arc::new(AppConfig {
        secret_key,
        base_fs_path,
    });

    let app = Router::new()
        .route("/images/{token}/{size}/{*img_path}", get(handle_resize))
        .with_state(config);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server is started in 127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap()
}

fn parse_dimensions(size: &str) -> Option<(u32, u32)> {
    let mut parts = size.split("x");
    let width = parts.next()?.parse::<u32>().ok()?;
    let height = parts.next()?.parse::<u32>().ok()?;

    Some((width, height))
}

async fn handle_resize(
    State(config): State<Arc<AppConfig>>,
    Path((token, size, img_path)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let expected_token = generate_token(&img_path, &config.secret_key);

    if token != expected_token {
        return Err(StatusCode::FORBIDDEN);
    }

    let (width, height) = parse_dimensions(&size).ok_or(StatusCode::BAD_REQUEST)?;

    let full_path = PathBuf::from(&config.base_fs_path).join(&img_path);
    if !full_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let output_bytes = tokio::task::spawn_blocking(move || {
        let img = image::open(&full_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let resized = img.resize(width, height, FilterType::Triangle);
        let mut buffer = Cursor::new(Vec::new());

        resized
            .write_to(&mut buffer, image::ImageFormat::Jpeg)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok::<Vec<u8>, StatusCode>(buffer.into_inner())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    Ok(([(header::CONTENT_TYPE, "image/jpeg")], output_bytes))
}

fn generate_token(path: &str, secret_key: &str) -> String {
    let mut mac = HmacSha1::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can accept a key of any length.");

    mac.update(path.as_bytes());

    let result = mac.finalize();
    let bytes = result.into_bytes();

    let b64 = BASE64_STANDARD.encode(bytes);

    let modified: String = b64
        .chars()
        .map(|c| match c {
            '+' => '-',
            '/' => '_',
            '=' => ',',
            other => other,
        })
        .collect();

    modified[..12].to_string()
}
