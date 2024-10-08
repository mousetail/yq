pub struct Lang {
    pub name: &'static str,
    pub bin_location: &'static str,
    pub plugin: &'static str,
    pub env: &'static [(&'static str, &'static str)],
    pub latest_version: &'static str,
}

pub const LANGS: &[Lang] = &[
    Lang {
        name: "nodejs",
        bin_location: "/bin/node",
        plugin: "https://github.com/asdf-vm/asdf-nodejs.git",
        env: &[],
        latest_version: "22.9.0",
    },
    Lang {
        name: "python",
        bin_location: "/bin/python",
        plugin: "https://github.com/asdf-community/asdf-python.git",
        env: &[("LD_LIBRARY_PATH", "/lang/lib")],
        latest_version: "3.12.0",
    },
];
