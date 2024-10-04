use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Json};
use common::RunLangOutput;
use sqlx::PgPool;

use crate::{
    auto_output_format::{AutoOutputFormat, Format},
    models::{
        challenge::Challenge,
        solutions::{NewSolution, Solution},
        InsertedId,
    },
    test_solution::test_solution,
};

pub async fn all_solutions(
    Path(challenge_id): Path<i32>,
    format: Format,
    Extension(pool): Extension<PgPool>,
) -> AutoOutputFormat<Vec<Solution>> {
    let sql = "SELECT id, language, version, challenge, code FROM solutions WHERE challenge=$1";
    let solutions = sqlx::query_as::<_, Solution>(&sql)
        .bind(challenge_id)
        .fetch_all(&pool)
        .await
        .unwrap();

    AutoOutputFormat::new(solutions, "index.html.jinja", format)
}

pub async fn get_solution(
    Path(challenge_id): Path<i32>,
    Path(id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Solution>, ()> {
    let sql =
        "SELECT id, language, version, challenge, code FROM solutions WHERE id=$1 AND challenge=$2"
            .to_string();

    let solution: Solution = sqlx::query_as(&sql)
        .bind(id)
        .bind(challenge_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| ())?;
    Ok(Json(solution))
}

#[axum::debug_handler]
pub async fn new_solution(
    Path(challenge_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
    Json(solution): Json<NewSolution>,
) -> Result<(StatusCode, Json<RunLangOutput>), ()> {
    let challenge = Challenge::get_by_id(&pool, challenge_id).await.unwrap();
    let test_result = test_solution(&solution, &challenge).await.unwrap();

    if test_result.tests.pass {
        let sql = "INSERT INTO solutions (language, version, challenge, code) values ($1, $2, $3, $4) RETURNING id";

        let InsertedId(_row) = sqlx::query_as(&sql)
            .bind(&solution.language)
            .bind(&solution.version)
            .bind(&challenge_id)
            .bind(&solution.code)
            .fetch_one(&pool)
            .await
            .map_err(|_| ())?;
        Ok((StatusCode::CREATED, Json(test_result)))
    } else {
        Ok((StatusCode::NOT_ACCEPTABLE, Json(test_result)))
    }
}
