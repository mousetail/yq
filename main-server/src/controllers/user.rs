use axum::{extract::Path, Extension};
use serde::Serialize;
use sqlx::{query_as, query_scalar, PgPool};

use crate::{
    auto_output_format::{AutoOutputFormat, Format},
    error::Error,
};

#[derive(Serialize)]
pub struct UserPageLeaderboardEntry {
    language: String,
    score: i32,
    challenge_id: i32,
    challenge_name: String,
}

#[derive(Serialize)]
pub struct UserInfo {
    user_name: String,
    solutions: Vec<UserPageLeaderboardEntry>,
}

pub async fn get_user(
    Path(id): Path<i32>,
    format: Format,
    Extension(pool): Extension<PgPool>,
) -> Result<AutoOutputFormat<UserInfo>, Error> {
    let user_name = query_scalar!("SELECT username FROM accounts WHERE id=$1", id)
        .fetch_optional(&pool)
        .await
        .map_err(Error::DatabaseError)?;
    let Some(user_name) = user_name else {
        return Err(Error::NotFound);
    };

    let solutions = query_as!(
        UserPageLeaderboardEntry,
        "SELECT solutions.language, solutions.score, solutions.challenge as challenge_id, challenges.name as challenge_name
        FROM solutions
        LEFT JOIN challenges ON challenges.id = solutions.challenge
        WHERE solutions.author=$1",
        id
    ).fetch_all(&pool).await
    .map_err(Error::DatabaseError)?;

    Ok(AutoOutputFormat::new(
        UserInfo {
            solutions,
            user_name,
        },
        "user.html.jinja",
        format,
    ))
}
