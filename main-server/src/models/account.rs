use axum::{
    async_trait,
    body::Body,
    extract::FromRequestParts,
    http::{request::Parts, Response},
    response::IntoResponse,
    Extension,
};
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::{prelude::FromRow, PgPool};
use tower_sessions::Session;

use crate::controllers::auth::ACCOUNT_ID_KEY;

#[derive(FromRow, Serialize)]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub avatar: String,
}

impl Account {
    pub async fn get_by_id(pool: &PgPool, id: i32) -> Option<Self> {
        sqlx::query_as("SELECT id, username, avatar from accounts where id=$1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap()
    }
}

#[derive(Debug)]
pub enum AccountFetchError {
    SessionLoadFailed,
    NoSession,
    NotLoggedIn,
    NoAccountFound,
    DatabaseLoadFailed,
}

impl IntoResponse for AccountFetchError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AccountFetchError::NotLoggedIn => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from(
                    r#"<h2>Not authorized</h2>
                    <p>You must be logged in to perform this action</p>

                    <a href="/login/github">Login</a>
                "#,
                ))
                .unwrap(),
            e => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "text/plain")
                .body(Body::from(println!("{e:#?}")))
                .unwrap(),
        }
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Account {
    type Rejection = AccountFetchError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AccountFetchError::SessionLoadFailed)?;
        let Extension(pool) = Extension::<PgPool>::from_request_parts(parts, state)
            .await
            .map_err(|_| AccountFetchError::DatabaseLoadFailed)?;

        if let Some(account_id) = session
            .get(ACCOUNT_ID_KEY)
            .await
            .map_err(|_| AccountFetchError::NoSession)?
        {
            if let Some(account) = Account::get_by_id(&pool, account_id).await {
                return Ok(account);
            }
            return Err(AccountFetchError::NoAccountFound);
        } else {
            return Err(AccountFetchError::NotLoggedIn);
        }
    }
}
