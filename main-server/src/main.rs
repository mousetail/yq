mod controllers;
mod models;

use auto_reload::restart;
use axum::{routing::get, Extension, Router};

use anyhow::Context;
use controllers::challenges::{all_challenges, get_challenge, new_challenge};
use sqlx::postgres::PgPoolOptions;
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let auto_reload = std::env::args().any(|k| k == "--reload");
    println!("Starting server.. {auto_reload}");

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
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    let server = axum::serve(listener, app);
    if auto_reload {
        server
            .with_graceful_shutdown(auto_reload::wait_reload())
            .await
            .unwrap();
    } else {
        server.await.unwrap();
    }
    println!("Reloading...");
    if auto_reload {
        restart().await;
    }
    Ok(())
}
