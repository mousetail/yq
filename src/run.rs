use std::{fs::OpenOptions, io::Write, path::PathBuf, process::Stdio};

use async_process::Command;
use serde::Serialize;

use crate::{
    cachemap::CacheMap,
    error::{RunLangError, RunProcessError},
    langs::{Lang, LANGS},
    Message,
};

#[derive(Serialize)]
pub struct RunLangOutput {
    #[serde(with = "serde_bytes")]
    stdout: Vec<u8>,
    #[serde(with = "serde_bytes")]
    stderr: Vec<u8>,
}

async fn install_plugin(lang: &Lang) -> Result<CacheMap<String, ()>, RunProcessError> {
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
    let output = Command::new("asdf")
        .args(["install", lang.name, &version])
        .stderr(Stdio::inherit())
        .output()
        .await?;

    if !output.status.success() {
        return Err(RunProcessError::NonZeroStatusCode(output.status.code()));
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

async fn run_lang(
    lang_name: String,
    version: String,
    code: String,
) -> Result<RunLangOutput, RunProcessError> {
    let lang = LANGS.iter().find(|k| k.name == lang_name).unwrap();

    let lang_folder = Command::new("asdf")
        .args(["where", lang.name, &version])
        .stderr(Stdio::inherit())
        .output()
        .await?;
    if !lang_folder.status.success() {
        return Err(RunProcessError::NonZeroStatusCode(
            lang_folder.status.code(),
        ));
    }

    let buff = PathBuf::from(String::from_utf8(lang_folder.stdout).unwrap().trim());

    let temp_directory = tempfile::TempDir::new()?;
    let path = temp_directory.path().join("file");
    let mut tmp_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    tmp_file.write_all(code.as_bytes())?;
    tmp_file.flush()?;

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
        ])
        .args(["--ro-bind"])
        .arg(path)
        .args(["/code", "--ro-bind"])
        .arg(buff)
        .args(["/lang"])
        .args(["--unshare-all", "--new-session"]);

    for (key, value) in lang.env {
        command.args(["--setenv", *key, *value]);
    }

    command.arg(lang.bin_location).arg("/code");

    let output = command.output().await?;

    let mut stdout = output.stdout;
    stdout.truncate(10000);
    let mut stderr = output.stderr;
    stderr.truncate(1000);

    Ok(RunLangOutput {
        stdout: stdout,
        stderr: stderr,
    })
}

pub async fn process_message(
    message: Message,
    lang_versions: &CacheMap<String, CacheMap<String, ()>>,
) -> Result<RunLangOutput, RunLangError> {
    install_lang(message.lang.clone(), &message.version, lang_versions)
        .await
        .map_err(|e| RunLangError::PluginInstallFailure(e))?;
    let output = run_lang(message.lang, message.version, message.code)
        .await
        .map_err(|k| RunLangError::RunLangError(k))?;
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
