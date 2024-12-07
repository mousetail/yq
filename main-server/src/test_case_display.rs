use std::borrow::Cow;

use common::{RunLangOutput, TestCase, TestPassState};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseDisplay {
    columns: Vec<Column>,
    title: Option<Cow<'static, str>>,
    status: TestPassState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    title: Option<Cow<'static, str>>,
    content: String,
}

impl TestCaseDisplay {
    pub fn from_test_case(test_case: TestCase) -> Self {
        let columns = match test_case.result_display {
            common::ResultDisplay::Empty => vec![],
            common::ResultDisplay::Text(e) => vec![Column {
                title: None,
                content: e,
            }],
            common::ResultDisplay::Diff { output, expected } => vec![
                Column {
                    title: Some(Cow::Borrowed("Output")),
                    content: output,
                },
                Column {
                    title: Some(Cow::Borrowed("Expected")),
                    content: expected,
                },
            ],
            common::ResultDisplay::Run {
                input,
                output,
                error,
            } => vec![
                Column {
                    title: Some(Cow::Borrowed("Input")),
                    content: input.unwrap_or_default(),
                },
                Column {
                    title: Some(Cow::Borrowed("Output")),
                    content: output,
                },
                Column {
                    title: Some(Cow::Borrowed("Error")),
                    content: error,
                },
            ],
        };

        TestCaseDisplay {
            columns,
            title: test_case.name.map(|e| Cow::Owned(e)),
            status: test_case.pass,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputDisplay {
    tests: Vec<TestCaseDisplay>,
    passed: bool,
    timed_out: bool,
    judge_error: Option<String>,
}

impl From<RunLangOutput> for OutputDisplay {
    fn from(value: RunLangOutput) -> Self {
        OutputDisplay {
            tests: value
                .tests
                .test_cases
                .into_iter()
                .map(|e| TestCaseDisplay::from_test_case(e))
                .collect(),
            passed: value.tests.pass,
            timed_out: value.timed_out,
            judge_error: (!value.stderr.is_empty()).then(|| value.stderr),
        }
    }
}
