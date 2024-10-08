use axum::{http::StatusCode, Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    auto_output_format::{AutoOutputFormat, Format},
    models::{
        challenge::{Challenge, NewChallenge},
        InsertedId,
    },
};

#[derive(Serialize)]
pub struct AllChallengesOutput {
    challenges: Vec<Challenge>,
}

pub async fn all_challenges(
    Extension(pool): Extension<PgPool>,
    format: Format,
) -> AutoOutputFormat<AllChallengesOutput> {
    let sql = "SELECT * FROM challenges";
    let challenges = sqlx::query_as::<_, Challenge>(sql)
        .fetch_all(&pool)
        .await
        .unwrap();

    AutoOutputFormat::new(
        AllChallengesOutput {
            challenges,
        },
        "home.html.jinja",
        format,
    )
}

#[axum::debug_handler]
pub async fn new_challenge(
    Extension(pool): Extension<PgPool>,
    Json(challenge): Json<NewChallenge>,
) -> Result<(StatusCode, Json<Challenge>), ()> {
    // if task.task.is_empty() {
    //     return Err(CustomError::BadRequest)
    // }
    let sql = "INSERT INTO challenges (name, judge, description) values ($1, $2, $3) RETURNING id";

    let InsertedId(row) = sqlx::query_as(sql)
        .bind(&challenge.name)
        .bind(&challenge.judge)
        .bind(&challenge.description)
        .fetch_one(&pool)
        .await
        .map_err(|_| ())?;

    Ok((StatusCode::CREATED, Json(Challenge { challenge, id: row })))
}
