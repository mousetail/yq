mod error;
mod langs;

use error::{RunLangError, RunProcessError};
use langs::LANGS;
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Serialize, Debug)]
pub enum Message {
    Install {
        lang: String,
        version: String,
    },
    Run {
        lang: String,
        version: String,
        code: String,
    },
}

fn install_lang(
    lang_name: String,
    version: String,
    versions: &mut HashMap<String, HashSet<String>>,
) -> Result<(), RunProcessError> {
    let lang = LANGS.iter().find(|k| k.name == lang_name).unwrap();

    let versions = match versions.entry(lang.name.to_owned()) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(vac) => {
            let plugin_install_output = Command::new("asdf")
                .args(["plugin", "add", lang.name, lang.plugin])
                .stderr(Stdio::inherit())
                .status()?;
            if !plugin_install_output.success() {
                return Err(RunProcessError::NonZeroStatusCode(
                    plugin_install_output.code(),
                ));
            }
            vac.insert(HashSet::new())
        }
    };

    if !versions.contains(&version) {
        let output = Command::new("asdf")
            .args(["install", lang.name, &version])
            .stderr(Stdio::inherit())
            .output()?;

        if !output.status.success() {
            return Err(RunProcessError::NonZeroStatusCode(output.status.code()));
        }

        versions.insert(version);
    }

    Ok(())
}

fn run_lang(lang_name: String, version: String, code: String) -> Result<(), RunProcessError> {
    let lang = LANGS.iter().find(|k| k.name == lang_name).unwrap();

    let lang_folder = Command::new("asdf")
        .args(["where", lang.name, &version])
        .stderr(Stdio::inherit())
        .output()?;
    if !lang_folder.status.success() {
        return Err(RunProcessError::NonZeroStatusCode(
            lang_folder.status.code(),
        ));
    }

    let mut buff = PathBuf::from(String::from_utf8(lang_folder.stdout).unwrap().trim());
    buff.push(lang.bin_location);

    let temp_directory = tempfile::TempDir::new()?;
    let path = temp_directory.path().join(".file");
    let mut tmp_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    tmp_file.write_all(code.as_bytes())?;
    tmp_file.flush()?;

    let _code_output = Command::new(buff)
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    Ok(())
}

fn process_message(
    message: Message,
    lang_versions: &mut HashMap<String, HashSet<String>>,
) -> Result<(), RunLangError> {
    match message {
        Message::Install { lang, version } => install_lang(lang, version, lang_versions)
            .map_err(|k| RunLangError::PluginInstallFailure(k)),
        Message::Run {
            lang,
            version,
            code,
        } => run_lang(lang, version, code).map_err(|k| RunLangError::RunLangError(k)),
    }
}

fn get_lang_versions() -> HashMap<String, HashSet<String>> {
    let output = Command::new("asdf")
        .args(["plugin", "list"])
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("Finding the list of plugins failed");
    }

    let output_text = String::from_utf8(output.stdout).unwrap();
    output_text
        .lines()
        .map(|line| {
            let parts = line.split_ascii_whitespace().collect::<Vec<_>>();
            let Some(name) = parts.first() else {
                panic!("bad output from asdf plugin list: {line} {parts:?}")
            };

            let versions = Command::new("asdf").args(["list", name]).output().unwrap();

            if !versions.status.success() {
                println!("Finding versions failed");
            }

            return (
                (*name).to_owned(),
                String::from_utf8(versions.stdout)
                    .unwrap()
                    .lines()
                    .map(|k| k.trim().to_owned())
                    .collect::<HashSet<_>>(),
            );
        })
        .collect()
}

fn main() {
    println!("Starting!");

    let mut lang_versions = get_lang_versions();
    println!("{lang_versions:?}");

    let messages = [
        Message::Install {
            lang: "nodejs".to_owned(),
            version: "17.3.0".to_owned(),
        },
        Message::Run {
            lang: "nodejs".to_owned(),
            version: "17.3.0".to_owned(),
            code: "console.log(\"Hello World!\");".to_owned(),
        },
    ];

    for message in messages {
        println!("processing message {message:?}");
        process_message(message, &mut lang_versions).unwrap();
    }

    let lang_versions = get_lang_versions();
    println!("{lang_versions:?}");
}
