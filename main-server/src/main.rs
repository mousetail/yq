mod auto_output_format;
mod controllers;
mod models;
mod test_solution;

use axum::{routing::get, Extension, Router};

use anyhow::Context;
use controllers::{
    challenges::{all_challenges, get_challenge, new_challenge},
    solution::{all_solutions, get_solution, new_solution},
};
use sqlx::postgres::PgPoolOptions;
use std::fs;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = fs::read_to_string(".env").unwrap();
    let env = env.lines().find(|k| k.contains("DATABASE_URL")).unwrap();
    let (key, database_url) = env.split_once('=').unwrap();

    assert_eq!(key.trim(), "DATABASE_URL");

    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .context("could not connect to database_url")?;

    let app = Router::new()
        .route("/", get(all_challenges).post(new_challenge))
        .route("/:id", get(get_challenge))
        .route("/:id/solutions", get(all_solutions).post(new_solution))
        .route("/:id/solutions/:solution_id", get(get_solution))
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
