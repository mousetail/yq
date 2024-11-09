pub mod langs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JudgeResult {
    pub pass: bool,
    pub test_cases: Vec<TestCase>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunLangOutput {
    pub tests: JudgeResult,
    pub stderr: String,
}

#[derive(Serialize, Deserialize)]
pub enum TestPassState {
    /// The test passed
    Pass,
    /// The test failed and caused the entire challenge to fail
    Fail,
    /// This particular test is only informational and has no effect on the pass/fail of the entire challenge
    Info,
    /// This test failed but failure does not nececairly mean the entire challenge failed. Can be used if, for example, you only need 5 out of 6
    /// Tests to pass
    Warning,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    #[serde(default)]
    pub name: Option<String>,
    pub pass: TestPassState,
    pub result_display: ResultDisplay,
}

#[derive(Serialize, Deserialize)]
pub enum ResultDisplay {
    Empty,
    Text(String),
    Diff { output: String, expected: String },
    Run { output: String, error: String },
}
