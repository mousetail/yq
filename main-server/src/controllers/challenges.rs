use axum::{
    body::Body,
    extract::Path,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Extension,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    auto_output_format::{AutoInput, AutoOutputFormat, Format},
    discord::post_new_challenge,
    error::Error,
    models::{
        account::Account,
        challenge::{Challenge, ChallengeWithAuthorInfo, NewChallenge, NewChallengeWithTests},
    },
    slug::Slug,
    solution_invalidation::notify_challenge_updated,
    test_solution::test_solution,
};

#[derive(Serialize)]
pub struct AllChallengesOutput {
    challenges: Vec<Challenge>,
}

pub async fn all_challenges(
    Extension(pool): Extension<PgPool>,
    format: Format,
) -> Result<AutoOutputFormat<AllChallengesOutput>, Error> {
    let sql = "SELECT * FROM challenges ORDER BY created_at DESC";
    let challenges = sqlx::query_as::<_, Challenge>(sql)
        .fetch_all(&pool)
        .await
        .map_err(Error::DatabaseError)?;

    Ok(AutoOutputFormat::new(
        AllChallengesOutput { challenges },
        "home.html.jinja",
        format,
    ))
}

#[axum::debug_handler]
pub async fn compose_challenge(
    id: Option<Path<i32>>,
    pool: Extension<PgPool>,
    format: Format,
) -> Result<AutoOutputFormat<NewChallenge>, Error> {
    Ok(AutoOutputFormat::new(
        match id {
            Some(Path(id)) => match ChallengeWithAuthorInfo::get_by_id(&pool, id)
                .await?
                .map(|d| d.challenge)
            {
                Some(m) => m.challenge,
                None => return Err(Error::NotFound),
            },
            None => NewChallenge::default(),
        },
        "submit_challenge.html.jinja",
        format,
    ))
}

pub async fn view_challenge(
    Path(id): Path<i32>,
    pool: Extension<PgPool>,
    format: Format,
) -> Result<AutoOutputFormat<NewChallenge>, Error> {
    Ok(AutoOutputFormat::new(
        match ChallengeWithAuthorInfo::get_by_id(&pool, id)
            .await?
            .map(|d| d.challenge)
        {
            Some(m) => m.challenge,
            None => return Err(Error::NotFound),
        },
        "view_challenge.html.jinja",
        format,
    ))
}

pub async fn new_challenge(
    id: Option<Path<i32>>,
    Extension(pool): Extension<PgPool>,
    account: Account,
    format: Format,
    AutoInput(challenge): AutoInput<NewChallenge>,
) -> Result<Response<Body>, Error> {
    let tests = test_solution(
        &challenge.example_code,
        "nodejs",
        "22.4.0",
        &challenge.judge,
    )
    .await
    .map_err(|_| Error::ServerError)?;

    if !tests.tests.pass {
        return Ok(AutoOutputFormat::new(
            NewChallengeWithTests {
                challenge,
                tests: Some(tests),
            },
            "submit_challenge.html.jinja",
            format,
        )
        .with_status(StatusCode::BAD_REQUEST)
        .into_response());
    }

    match id {
        None => {
            let row = sqlx::query_scalar!(r"INSERT INTO challenges (name, judge, description, author) values ($1, $2, $3, $4) RETURNING id",
                challenge.name,
                challenge.judge,
                challenge.description,
                account.id
            )
                .fetch_one(&pool)
                .await
                .map_err(Error::DatabaseError)?;

            let redirect =
                Redirect::temporary(&format!("/challenge/{row}/{}/edit", Slug(&challenge.name)))
                    .into_response();
            tokio::spawn(post_new_challenge(account, challenge, row));

            Ok(redirect)
        }
        Some(Path(id)) => {
            sqlx::query!(
                r"UPDATE challenges SET name=$1, judge=$2, description=$3, example_code=$4 WHERE id=$5",
                challenge.name,
                challenge.judge,
                challenge.description,
                challenge.example_code,
                id
            )
            .execute(&pool)
            .await
            .unwrap();

            // Tells the solution invalidator task to re-check all solutions
            notify_challenge_updated();

            Ok(AutoOutputFormat::new(
                NewChallengeWithTests {
                    challenge,
                    tests: Some(tests),
                },
                "submit_challenge.html.jinja",
                format,
            )
            .into_response())
        }
    }
}
