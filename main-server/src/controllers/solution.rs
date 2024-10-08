use axum::{extract::Path, http::StatusCode, Extension, Json};
use common::{langs::LANGS, RunLangOutput};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    auto_output_format::{AutoInput, AutoOutputFormat, Format},
    error::Error,
    models::{
        account::Account,
        challenge::Challenge,
        solutions::{LeaderboardEntry, NewSolution, Solution},
        InsertedId,
    },
    test_solution::test_solution,
};

#[derive(Serialize)]
pub struct AllSolutionsOutput {
    challenge: Challenge,
    leaderboard: Vec<LeaderboardEntry>,
    tests: Option<RunLangOutput>,
}

pub async fn all_solutions(
    Path((challenge_id, language_name)): Path<(i32, String)>,
    format: Format,
    Extension(pool): Extension<PgPool>,
) -> Result<AutoOutputFormat<AllSolutionsOutput>, Error> {
    let leaderboard = LeaderboardEntry::get_leadeboard_for_challenge_and_language(
        &pool,
        challenge_id,
        &language_name,
    )
    .await;

    let challenge = Challenge::get_by_id(&pool, challenge_id).await?;

    Ok(AutoOutputFormat::new(
        AllSolutionsOutput {
            challenge,
            leaderboard,
            tests: None,
        },
        "challenge.html.jinja",
        format,
    ))
}

pub async fn get_solution(
    Path((challenge_id, _language_name, solution_id)): Path<(i32, String, i32)>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Solution>, ()> {
    let sql =
        "SELECT id, language, version, challenge, code FROM solutions WHERE id=$1 AND challenge=$2"
            .to_string();

    let solution: Solution = sqlx::query_as(&sql)
        .bind(solution_id)
        .bind(challenge_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| ())?;
    Ok(Json(solution))
}

#[axum::debug_handler]
pub async fn new_solution(
    Path((challenge_id, language_name)): Path<(i32, String)>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    format: Format,
    AutoInput(solution): AutoInput<NewSolution>,
) -> Result<AutoOutputFormat<AllSolutionsOutput>, Error> {
    let challenge = Challenge::get_by_id(&pool, challenge_id).await.unwrap();

    let version = LANGS
        .iter()
        .find(|i| i.name == language_name)
        .ok_or(Error::NotFound)?
        .latest_version;

    let test_result = test_solution(&solution, &language_name, version, &challenge)
        .await
        .unwrap();

    if test_result.tests.pass {
        let sql = "INSERT INTO solutions (language, version, challenge, code, author, score) values ($1, $2, $3, $4, $5, $6) RETURNING id";

        let InsertedId(_row) = sqlx::query_as(sql)
            .bind(&language_name)
            .bind(version)
            .bind(challenge_id)
            .bind(&solution.code)
            .bind(account.id)
            .bind(solution.code.len() as i32)
            .fetch_one(&pool)
            .await
            .map_err(|_| Error::ServerError)?;
        Ok(AutoOutputFormat::new(
            AllSolutionsOutput {
                challenge,
                leaderboard: LeaderboardEntry::get_leadeboard_for_challenge_and_language(
                    &pool,
                    challenge_id,
                    &language_name,
                )
                .await,
                tests: Some(test_result),
            },
            "challenge.html.jinja",
            format,
        )
        .with_status(StatusCode::BAD_REQUEST))
    } else {
        Ok(AutoOutputFormat::new(
            AllSolutionsOutput {
                challenge,
                leaderboard: LeaderboardEntry::get_leadeboard_for_challenge_and_language(
                    &pool,
                    challenge_id,
                    &language_name,
                )
                .await,
                tests: Some(test_result),
            },
            "challenge.html.jinja",
            format,
        )
        .with_status(StatusCode::CREATED))
    }
}
