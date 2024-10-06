use axum::{body::Body, http::Response, response::IntoResponse};
use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    NotFound,
    #[allow(unused)]
    ServerError,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap(),
            Error::ServerError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
        }
    }
}
