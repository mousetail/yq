use common::RunLangOutput;
use serde::Serialize;

use crate::models::{challenge::Challenge, solutions::NewSolution};

#[derive(Serialize)]
struct TestRunnerRequest<'a> {
    lang: &'a str,
    version: &'a str,
    code: &'a str,
    judge: &'a str,
}

pub async fn test_solution(
    solution: &NewSolution,
    challenge: &Challenge,
) -> reqwest::Result<RunLangOutput> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000")
        .json(&TestRunnerRequest {
            lang: &solution.language,
            version: &solution.version,
            code: &solution.code,
            judge: &challenge.challenge.judge,
        })
        .send()
        .await?
        .error_for_status()?
        .json::<RunLangOutput>()
        .await?;

    Ok(resp)
}
