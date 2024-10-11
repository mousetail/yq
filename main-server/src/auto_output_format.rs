use std::{convert::Infallible, sync::OnceLock};

use axum::{
    async_trait,
    body::Body,
    extract::{
        rejection::{FormRejection, JsonRejection},
        FromRequest, FromRequestParts,
    },
    http::{request::Parts, Response},
    response::IntoResponse,
    Form, Json,
};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use tera::{escape_html, Context, Tera};

use crate::models::account::Account;

#[derive(Serialize)]
pub struct HtmlContext {
    account: Option<Account>,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for HtmlContext {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let account = Account::from_request_parts(parts, state).await.ok();

        return Ok(HtmlContext { account });
    }
}

pub enum Format {
    Json,
    Html(Box<HtmlContext>),
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Format {
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
            || parts.headers.get("accept").is_some_and(|d| {
                let bytes = d.as_bytes();
                bytes.eq_ignore_ascii_case(b"application/json")
            })
        {
            return Ok(Format::Json);
        } else {
            return Ok(Format::Html(Box::new(
                HtmlContext::from_request_parts(parts, state).await?,
            )));
        }
    }
}

pub struct AutoOutputFormat<T: Serialize> {
    data: T,
    format: Format,
    template: &'static str,
    status: StatusCode,
}

impl<T: Serialize> AutoOutputFormat<T> {
    pub fn new(data: T, template: &'static str, format: Format) -> Self {
        AutoOutputFormat {
            data,
            format,
            template,
            status: StatusCode::OK,
        }
    }

    pub fn with_status(self, status: StatusCode) -> Self {
        AutoOutputFormat { status, ..self }
    }

    fn create_html_response(
        data: T,
        template: &'static str,
        status: StatusCode,
        html_context: &HtmlContext,
    ) -> axum::response::Response {
        let value = TERA.get_or_init(|| Tera::new("templates/**/*.jinja"));

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
        context.insert("object", &data);
        context.insert("account", &html_context.account);

        let html = tera.render(template, &context).unwrap();
        Response::builder()
            .status(status)
            .body(axum::body::Body::from(html))
            .unwrap()
    }

    fn create_json_response(&self) -> axum::response::Response {
        let mut response = Json(&self.data).into_response();
        *response.status_mut() = self.status;
        response
    }
}

static TERA: OnceLock<tera::Result<Tera>> = OnceLock::new();

impl<T: Serialize> IntoResponse for AutoOutputFormat<T> {
    fn into_response(self) -> axum::response::Response {
        match self.format {
            Format::Html(context) => {
                Self::create_html_response(self.data, self.template, self.status, &context)
            }
            Format::Json => self.create_json_response(),
        }
    }
}

pub struct AutoInput<T: DeserializeOwned>(pub T);

pub enum AutoInputRejection {
    JsonRejection(JsonRejection),
    FormRejection(FormRejection),
    BadContentType,
}

impl IntoResponse for AutoInputRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            AutoInputRejection::JsonRejection(json_rejection) => json_rejection.into_response(),
            AutoInputRejection::FormRejection(form_rejection) => form_rejection.into_response(),
            AutoInputRejection::BadContentType => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "text/plain")
                .body(Body::from("Excpected a content type"))
                .unwrap(),
        }
    }
}

impl From<JsonRejection> for AutoInputRejection {
    fn from(value: JsonRejection) -> Self {
        AutoInputRejection::JsonRejection(value)
    }
}

impl From<FormRejection> for AutoInputRejection {
    fn from(value: FormRejection) -> Self {
        AutoInputRejection::FormRejection(value)
    }
}

#[async_trait]
impl<T: DeserializeOwned, S: Sync + Send> FromRequest<S> for AutoInput<T> {
    type Rejection = AutoInputRejection;

    async fn from_request(
        request: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let content_type = request.headers().get("content-type");

        if content_type.is_some_and(|b| b.as_bytes().eq_ignore_ascii_case(b"application/json")) {
            let Json(value) = Json::<T>::from_request(request, state).await?;
            return Ok(AutoInput(value));
        } else if content_type.is_some_and(|b| {
            b.as_bytes()
                .eq_ignore_ascii_case(b"application/x-www-form-urlencoded")
        }) {
            let Form(value) = Form::<T>::from_request(request, state).await?;
            return Ok(AutoInput(value));
        } else {
            return Err(AutoInputRejection::BadContentType);
        }
    }
}
