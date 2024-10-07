mod auto_output_format;
mod controllers;
mod error;
mod models;
mod session;
mod test_solution;

use axum::{routing::get, Extension, Router};

use anyhow::Context;
use controllers::{
    auth::{github_callback, github_login},
    challenges::{all_challenges, get_challenge, new_challenge},
    solution::{all_solutions, get_solution, new_solution},
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::signal;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup .env
    #[cfg(debug_assertions)]
    {
        dotenvy::from_filename(".env.local")?;
        dotenvy::dotenv()?;
    }

    // Setup Tracking Subscriber
    tracing_subscriber::fmt()
        .log_internal_errors(true)
        // .with_span_events(FmtSpan::FULL)
        .init();

    // Setup SQLX
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&env::var("DATABASE_URL").expect("Missing .env var: DATABASE_URL"))
        .await
        .context("could not connect to database_url")?;

    // Setup Sessions
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_name("yq_session_store_id")
        .with_expiry(Expiry::OnInactivity(Duration::hours(10)));

    let app = Router::new()
        .route("/", get(all_challenges).post(new_challenge))
        .route("/:id", get(get_challenge))
        .route(
            "/:id/solutions/:language",
            get(all_solutions).post(new_solution),
        )
        .route("/:id/solutions/:language/:solution_id", get(get_solution))
        .route("/login/github", get(github_login))
        .route("/callback/github", get(github_callback))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(Extension(pool))
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    println!("Session store final state {session_store:?}");
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
