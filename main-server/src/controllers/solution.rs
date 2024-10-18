use axum::{extract::Path, http::StatusCode, Extension, Json};
use common::{langs::LANGS, RunLangOutput};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    auto_output_format::{AutoInput, AutoOutputFormat, Format},
    error::Error,
    models::{
        account::Account,
        challenge::{Challenge, ChallengeWithAuthorInfo},
        solutions::{Code, LeaderboardEntry, NewSolution, Solution},
    },
    test_solution::test_solution,
};

#[derive(Serialize)]
pub struct AllSolutionsOutput {
    challenge: ChallengeWithAuthorInfo,
    leaderboard: Vec<LeaderboardEntry>,
    tests: Option<RunLangOutput>,
    code: Option<String>,
}

pub async fn all_solutions(
    Path((challenge_id, language_name)): Path<(i32, String)>,
    format: Format,
    account: Option<Account>,
    Extension(pool): Extension<PgPool>,
) -> Result<AutoOutputFormat<AllSolutionsOutput>, Error> {
    let leaderboard = LeaderboardEntry::get_leadeboard_for_challenge_and_language(
        &pool,
        challenge_id,
        &language_name,
    )
    .await;

    let challenge = ChallengeWithAuthorInfo::get_by_id(&pool, challenge_id)
        .await?
        .ok_or(Error::NotFound)?;
    let code = match account {
        Some(account) => {
            Code::get_best_code_for_user(&pool, account.id, challenge_id, &language_name).await
        }
        None => None,
    };

    Ok(AutoOutputFormat::new(
        AllSolutionsOutput {
            challenge,
            leaderboard,
            tests: None,
            code: code.map(|d| d.code),
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

pub async fn new_solution(
    Path((challenge_id, language_name)): Path<(i32, String)>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    format: Format,
    AutoInput(solution): AutoInput<NewSolution>,
) -> Result<AutoOutputFormat<AllSolutionsOutput>, Error> {
    let challenge = ChallengeWithAuthorInfo::get_by_id(&pool, challenge_id)
        .await?
        .ok_or(Error::NotFound)
        .unwrap();

    let version = LANGS
        .iter()
        .find(|i| i.name == language_name)
        .ok_or(Error::NotFound)?
        .latest_version;

    let test_result = test_solution(
        &solution.code,
        &language_name,
        version,
        &challenge.challenge.challenge.judge,
    )
    .await
    .unwrap();
    let previous_code =
        Code::get_best_code_for_user(&pool, account.id, challenge_id, &language_name).await;

    let status = if test_result.tests.pass {
        match previous_code {
            None => {
                sqlx::query!(
                    "INSERT INTO solutions (language, version, challenge, code, author, score) values ($1, $2, $3, $4, $5, $6)",
                    language_name,
                    version,
                    challenge_id,
                    solution.code,
                    account.id,
                    solution.code.len() as i32,
                )
                .execute(&pool)
                .await
                .map_err(|_| Error::ServerError)?;

                StatusCode::CREATED
            }
            Some(w) if w.score >= solution.code.len() as i32 => {
                sqlx::query!(
                    "UPDATE solutions SET 
                        code=$1,
                        score=$2
                    WHERE id=$3",
                    solution.code,
                    solution.code.len() as i32,
                    w.id
                )
                .execute(&pool)
                .await
                .map_err(|_| Error::ServerError)?;

                StatusCode::CREATED
            }
            Some(_) => StatusCode::OK,
        }
    } else {
        StatusCode::BAD_REQUEST
    };
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
            code: Some(solution.code),
        },
        "challenge.html.jinja",
        format,
    )
    .with_status(status))
}
