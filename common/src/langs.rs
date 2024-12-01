use phf::phf_map;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Lang {
    pub name: &'static str,
    pub compile_command: &'static [&'static str],
    pub run_command: &'static [&'static str],
    pub plugin: &'static str,
    pub env: &'static [(&'static str, &'static str)],
    pub install_env: &'static [(&'static str, &'static str)],
    pub latest_version: &'static str,
    pub icon: &'static str,
}

pub const LANGS: phf::Map<&'static str, Lang> = phf_map! {
    "nodejs" => Lang {
        name: "nodejs",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/node", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-vm/asdf-nodejs.git",
        env: &[],
        install_env: &[],
        latest_version: "22.9.0",
        icon: "nodejs.svg"
    },
    "deno" => Lang {
        name: "deno",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/deno", "--allow-write=/tmp", "--allow-run", "--allow-read", "${FILE_LOCATION}"],
        //run_command: &["/usr/bin/env"],
        plugin: "https://github.com/asdf-community/asdf-deno.git",
        env: &[
            ("RUST_BACKTRACE", "1"),
            ("NO_COLOR", "1")
        ],
        install_env: &[],
        latest_version: "2.0.6",
        icon: "deno.svg"
    },
    "python" => Lang {
        name: "python",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/python", "${FILE_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-python.git",
        env: &[("LD_LIBRARY_PATH", "/lang/lib")],
        install_env: &[],
        latest_version: "3.12.0",
        icon: "python.svg"
    },
    "rust" => Lang {
        name: "rust",
        compile_command: &["${LANG_LOCATION}/bin/rustc", "${FILE_LOCATION}", "-o", "${OUTPUT_LOCATION}"],
        run_command: &["${OUTPUT_LOCATION}"],
        plugin: "https://github.com/asdf-community/asdf-rust.git",
        env: &[
            ("LD_LIBRARY_PATH", "/lang/lib:/lib"),
            ("PATH", "/usr/bin:/bin")
        ],
        install_env: &[(
            "RUST_WITHOUT",
            "rust-docs,rust-docs-json-preview,cargo,rustfmt-preview,rls-preview,rust-analyzer-preview,llvm-tools-preview,clippy-preview,rust-analysis-x86_64-unknown-linux-gnu,llvm-bitcode-linker-preview"
        )],
        latest_version: "1.82.0",
        icon: "rust.svg"
    },
    "vyxal" => Lang {
        name: "vyxal",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/vyxal2", "${FILE_LOCATION}", "'□'"],
        plugin: "https://github.com/lyxal/vyxasdf.git",
        env: &[],
        install_env: &[],
        latest_version: "2.22.4.3",
        icon: "vyxal.svg"
    },
    "tinyapl" => Lang {
        name: "tinyapl",
        compile_command: &[],
        run_command: &["${LANG_LOCATION}/bin/tinyapl", "${FILE_LOCATION}"],
        plugin: "https://github.com/RubenVerg/asdf-tinyapl.git",
        env: &[],
        install_env: &[],
        latest_version: "0.11.1.0",
	icon: "tinyapl.svg"
    },
};
