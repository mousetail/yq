mod auto_output_format;
mod controllers;
mod discord;
mod error;
mod markdown;
mod models;
mod test_solution;
mod vite;

use axum::{routing::get, Extension, Router};

use anyhow::Context;
use controllers::{
    auth::{github_callback, github_login},
    challenges::{all_challenges, compose_challenge, new_challenge},
    solution::{all_solutions, get_solution, new_solution},
    user::get_user,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::ExpiredDeletion;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use tower_sessions_file_store::FileSessionStorage;

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
    let session_store = FileSessionStorage::new();
    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_name("yq_session_store_id")
        .with_expiry(Expiry::OnInactivity(Duration::days(360)));
    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60 * 60)),
    );

    let app = Router::new()
        .route("/", get(all_challenges))
        .nest_service(
            "/ts/runner-lib.d.ts",
            ServeFile::new("scripts/build/runner-lib.d.ts"),
        )
        .nest_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .nest_service("/favicon.ico", ServeFile::new("static/favicon.svg"))
        .route("/challenge", get(compose_challenge).post(new_challenge))
        .route("/challenge/:id", get(compose_challenge).post(new_challenge))
        .route("/login/github", get(github_login))
        .route("/callback/github", get(github_callback))
        .route("/user/:id", get(get_user))
        .route("/:id/:language", get(all_solutions).post(new_solution))
        .route("/:id/:language/:solution_id", get(get_solution))
        .nest_service("/static", ServeDir::new("static"))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(Extension(pool))
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind(&format!(
        "{}:{}",
        env::var("YQ_HOST").expect("Expcted YQ_HOST var to be set"),
        env::var("YQ_PORT").expect("Excpected YQ_PORT var to be set")
    ))
    .await
    .unwrap();

    if let Ok(addr) = listener.local_addr() {
        println!("Listening on http://{addr:?}");
    }

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
