use std::{cell::OnceCell, collections::HashMap, convert::Infallible, sync::OnceLock};

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap, Response},
    middleware::FromExtractorLayer,
    response::IntoResponse,
};
use serde::Serialize;
use tera::{escape_html, Context, Tera};

pub enum Format {
    Json,
    Html,
}

#[async_trait]
impl<S> FromRequestParts<S> for Format {
    #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A rejection is"]
    #[doc = " a kind of error that can be converted into a response."]
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if parts
            .uri
            .path_and_query()
            .unwrap()
            .as_str()
            .ends_with(".json")
        {
            return Ok(Format::Json);
        } else if parts.headers.get("accept").is_some_and(|d| {
            let bytes = d.as_bytes();
            bytes.eq_ignore_ascii_case(b"application/json")
        }) {
            return Ok(Format::Json);
        } else {
            return Ok(Format::Html);
        }
    }
}

pub struct AutoOutputFormat<T: Serialize> {
    data: T,
    format: Format,
    template: &'static str,
}

impl<T: Serialize> AutoOutputFormat<T> {
    pub fn new(data: T, template: &'static str, format: Format) -> Self {
        AutoOutputFormat {
            data,
            format,
            template,
        }
    }

    fn create_html_response(&self) -> axum::response::Response {
        let value = (&TERA).get_or_init(|| {
            let tera = Tera::new("templates/**/*.jinja");
            return tera;
        });

        let tera = match value.as_ref() {
            Ok(tera) => tera,
            Err(e) => {
                return Response::builder()
                    .status(500)
                    .body(axum::body::Body::from(format!(
                        "<h1>Error Initializing Template Engine</h1>
                        <pre>{:?}</pre>
                ",
                        escape_html(&format!("{e:#?}"))
                    )))
                    .unwrap();
            }
        };

        let mut context = Context::new();
        context.insert("data", &self.data);

        let html = tera.render(&self.template, &context).unwrap();
        return Response::builder()
            .status(200)
            .body(axum::body::Body::from(html))
            .unwrap();
    }
}

static TERA: OnceLock<tera::Result<Tera>> = OnceLock::new();

impl<T: Serialize> IntoResponse for AutoOutputFormat<T> {
    fn into_response(self) -> axum::response::Response {
        self.create_html_response()
    }
}
