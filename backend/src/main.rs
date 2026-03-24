use axum::{routing::get, Router};
use hello_world_shared::HealthResponse;
use std::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:3000")?;
    println!("Server running on http://localhost:3000");
    axum::serve(tokio::net::TcpListener::from_std(listener)?, app).await?;
    Ok(())
}

async fn health_check() -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "OK".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
