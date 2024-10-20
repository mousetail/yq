use std::{path::PathBuf, process::Stdio};

use async_process::Command;
use common::{
    langs::{Lang, LANGS},
    JudgeResult, RunLangOutput, TestCase,
};
use futures_util::AsyncWriteExt;
use serde::{Deserialize, Serialize};

use crate::{
    cachemap::CacheMap,
    error::{RunLangError, RunProcessError},
    Message,
};

async fn install_plugin(lang: &Lang) -> Result<CacheMap<String, ()>, RunProcessError> {
    println!("Installing language version {}", lang.name);
    let plugin_install_output = Command::new("asdf")
        .args(["plugin", "add", lang.name, lang.plugin])
        .stderr(Stdio::inherit())
        .status()
        .await?;
    if !plugin_install_output.success() {
        return Err(RunProcessError::NonZeroStatusCode(
            plugin_install_output.code(),
        ));
    }
    Ok(CacheMap::new())
}

async fn install_language_version(lang: &Lang, version: &str) -> Result<(), RunProcessError> {
    println!("Installing language version {} {}", lang.name, version);
    let status = Command::new("asdf")
        .args(["install", lang.name, version])
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        return Err(RunProcessError::NonZeroStatusCode(status.code()));
    }
    Ok(())
}

async fn install_lang(
    lang_name: String,
    version: &str,
    versions: &CacheMap<String, CacheMap<String, ()>>,
) -> Result<(), RunProcessError> {
    let lang = LANGS.iter().find(|k| k.name == lang_name).unwrap();

    let lang_version_token = versions.get(lang.name.to_owned());
    let lang_versions = lang_version_token
        .get_or_try_init(|| install_plugin(lang))
        .await?;

    let specific_version_token = lang_versions.get(version.to_owned());

    let _specific_version = specific_version_token
        .get_or_try_init(|| install_language_version(lang, version))
        .await?;

    Ok(())
}

async fn get_lang_directory(lang: &Lang, version: &str) -> Result<PathBuf, RunProcessError> {
    let lang_folder = Command::new("asdf")
        .args(["where", lang.name, version])
        .stderr(Stdio::inherit())
        .output()
        .await?;
    if !lang_folder.status.success() {
        return Err(RunProcessError::NonZeroStatusCode(
            lang_folder.status.code(),
        ));
    }

    let buff = PathBuf::from(String::from_utf8(lang_folder.stdout).unwrap().trim());
    Ok(buff)
}

async fn run_lang(
    lang_name: &str,
    version: &str,
    code: &str,
    judge: &str,
    judge_lang: &str,
    judge_version: &str,
) -> Result<RunLangOutput, RunProcessError> {
    let lang = LANGS.iter().find(|k| k.name == lang_name).unwrap();
    let judge_lang = LANGS.iter().find(|k| k.name == judge_lang).unwrap();

    let code_lang_folder = get_lang_directory(lang, version).await?;
    let judge_lang_folder = get_lang_directory(judge_lang, judge_version).await?;

    // let temp_directory = tempfile::TempDir::new()?;
    // let path = temp_directory.path().join("file");
    // let mut tmp_file = OpenOptions::new()
    //     .create_new(true)
    //     .write(true)
    //     .open(&path)?;
    // tmp_file.write_all(code.as_bytes())?;
    // tmp_file.flush()?;
    println!("Running code in lang");

    let mut command = Command::new("bwrap");
    command
        .args([
            // "--clearenv",
            // "--hostname",
            // "yq",
            "--ro-bind",
            "/bin",
            "/bin",
            "--chdir",
            "/",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/etc/alternatives",
            "/etc/alternatives",
            "--tmpfs",
            "/tmp",
        ])
        .args(["--ro-bind"])
        .arg(code_lang_folder)
        .args(["/lang"])
        .args(["--ro-bind"])
        .arg(judge_lang_folder)
        .arg("/judge")
        .args(["--ro-bind", "/scripts", "/scripts"])
        .args(["--unshare-all", "--new-session"]);

    for (key, value) in judge_lang.env {
        command.args(["--setenv", *key, *value]);
    }

    command
        .args(
            judge_lang
                .run_command
                .iter()
                .map(|k| {
                    k.replace("${LANG_LOCATION}", "/judge")
                        .replace("${FILE_LOCATION}", "/scripts/runner.js")
                })
                .collect::<Vec<_>>(),
        )
        .stdout(Stdio::piped())
        .stdin(Stdio::piped());
    // .args([&format!("/lang/{}", lang.bin_location), code as &str, judge]);

    let mut child = command.spawn()?;
    let Some(stdin) = &mut child.stdin else {
        panic!("Child stdin should exist");
    };

    #[derive(Serialize)]
    struct RunnerInput<'a> {
        lang: &'a Lang,
        code: &'a str,
        judge: &'a str,
    }

    let data = serde_json::to_string(&RunnerInput { lang, code, judge })
        .map_err(|e| RunProcessError::SerializationFailed(e))?;
    stdin.write_all(data.as_bytes()).await?;

    let output = child.output().await?;

    let mut stdout = output.stdout;
    stdout.truncate(10000);
    let mut stderr = output.stderr;
    stderr.truncate(1000);

    Ok(RunLangOutput {
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        tests: parse_output(&stdout),
    })
}

#[derive(Serialize, Deserialize)]
pub struct FinalVerdict {
    pass: bool,
}

fn parse_output(out: &[u8]) -> JudgeResult {
    let mut test_cases = vec![];
    let mut pass = false;
    for line in out.split(|&k| k == b'\n') {
        if line.is_empty() {
            continue;
        }
        match serde_json::from_slice::<TestCase>(line) {
            Ok(test_case) => test_cases.push(test_case),
            Err(_) => match serde_json::from_slice::<FinalVerdict>(line) {
                Ok(FinalVerdict { pass: new_pass }) => pass = new_pass,
                Err(_e) => test_cases.push(TestCase {
                    name: Some("Judge Debug Message".to_owned()),
                    pass: common::TestPassState::Info,
                    result_display: common::ResultDisplay::Text(
                        String::from_utf8_lossy(line).to_string(),
                    ),
                    error: None,
                }),
            },
        }
    }

    JudgeResult { pass, test_cases }
}

pub async fn process_message(
    message: Message,
    lang_versions: &CacheMap<String, CacheMap<String, ()>>,
) -> Result<RunLangOutput, RunLangError> {
    // Runner Lang
    install_lang("nodejs".to_owned(), "22.9.0", lang_versions)
        .await
        .map_err(RunLangError::PluginInstallFailure)?;

    install_lang(message.lang.clone(), &message.version, lang_versions)
        .await
        .map_err(RunLangError::PluginInstallFailure)?;
    let output = run_lang(
        &message.lang,
        &message.version,
        &message.code,
        &message.judge,
        "nodejs",
        "22.9.0",
    )
    .await
    .map_err(RunLangError::RunLangError)?;
    Ok(output)
}

async fn get_versions_for_language(line: &str) -> (String, CacheMap<String, ()>) {
    let parts = line.split_ascii_whitespace().collect::<Vec<_>>();
    let Some(name) = parts.first() else {
        panic!("bad output from asdf plugin list: {line} {parts:?}")
    };

    let versions = Command::new("asdf")
        .args(["list", name])
        .output()
        .await
        .unwrap();

    if !versions.status.success() {
        println!("Finding versions failed");
    }

    return (
        (*name).to_owned(),
        String::from_utf8(versions.stdout)
            .unwrap()
            .lines()
            .map(|k| (k.trim().to_owned(), ()))
            .collect::<CacheMap<_, ()>>(),
    );
}

pub async fn get_lang_versions() -> CacheMap<String, CacheMap<String, ()>> {
    let output = Command::new("asdf")
        .args(["plugin", "list"])
        .output()
        .await
        .unwrap();
    if !output.status.success() {
        panic!("Finding the list of plugins failed");
    }

    let output_text = String::from_utf8(output.stdout).unwrap();
    futures_util::future::join_all(output_text.lines().map(get_versions_for_language))
        .await
        .into_iter()
        .collect()
}
