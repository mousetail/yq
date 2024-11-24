use axum::{body::Body, http::Response, response::IntoResponse};
use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    NotFound,
    #[allow(clippy::enum_variant_names)]
    ServerError,
    DatabaseError(sqlx::Error),
    OauthError(OauthError),
    RunLangError(String),
    PermissionDenied(&'static str),
}

#[derive(Debug)]
pub enum OauthError {
    TokenExchangeFailed,
    UserInfoFetchFailed,
    DeserializationFailed,
    CsrfValidationFailed,
}

impl IntoResponse for OauthError {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "Text/Plain")
            .body(Body::from(format!("{self:?}")))
            .unwrap()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(
                    r#"<h2>Not Found<h2>
                    <a href="/">Back to Home</a>
                "#,
                ))
                .unwrap(),
            Error::ServerError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
            Error::DatabaseError(e) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "Database Error: <pre>{}</pre>",
                    tera::escape_html(&format!("{e:#?}"))
                )))
                .unwrap(),
            Error::OauthError(oauth_error) => oauth_error.into_response(),
            Error::RunLangError(s) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!(
                    "<h2>Lang Runner Error</h2><pre>{}</pre>",
                    tera::escape_html(&s)
                )))
                .unwrap(),
            Error::PermissionDenied(e) => Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(format!(
                    "<h2>Not Authorized</h2>
                    <p>{}</p>",
                    tera::escape_html(e)
                )))
                .unwrap(),
        }
    }
}
