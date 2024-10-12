use common::RunLangOutput;
use serde::Serialize;


#[derive(Serialize)]
struct TestRunnerRequest<'a> {
    lang: &'a str,
    version: &'a str,
    code: &'a str,
    judge: &'a str,
}

pub async fn test_solution(
    code: &str,
    language: &str,
    version: &str,
    judge: &str,
) -> reqwest::Result<RunLangOutput> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000")
        .json(&TestRunnerRequest {
            lang: language,
            version,
            code,
            judge,
        })
        .send()
        .await?
        .error_for_status()?
        .json::<RunLangOutput>()
        .await?;

    Ok(resp)
}
