use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use sqlx::PgPool;

use crate::models::challenge::{Challenge, InsertedId, NewChallenge};

pub async fn all_challenges(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let sql = "SELECT * FROM challenges";
    let challenges = sqlx::query_as::<_, Challenge>(&sql)
        .fetch_all(&pool)
        .await
        .unwrap();

    (StatusCode::OK, Json(challenges))
}

pub async fn get_challenge(
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Challenge>, ()> {
    let sql = "SELECT * FROM challenges WHERE id=$1".to_string();

    let challenge: Challenge = sqlx::query_as(&sql)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| ())?;

    Ok(Json(challenge))
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

    let InsertedId(row) = sqlx::query_as(&sql)
        .bind(&challenge.name)
        .bind(&challenge.judge)
        .bind(&challenge.description)
        .fetch_one(&pool)
        .await
        .map_err(|_| ())?;

    Ok((StatusCode::CREATED, Json(Challenge { challenge, id: row })))
}
