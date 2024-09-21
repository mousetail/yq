pub struct Lang {
    pub name: &'static str,
    pub bin_location: &'static str,
    pub plugin: &'static str,
    pub env: &'static [(&'static str, &'static str)],
}

pub const LANGS: &'static [Lang] = &[
    Lang {
        name: "nodejs",
        bin_location: "/lang/bin/node",
        plugin: "https://github.com/asdf-vm/asdf-nodejs.git",
        env: &[],
    },
    Lang {
        name: "python",
        bin_location: "/lang/bin/python",
        plugin: "https://github.com/asdf-community/asdf-python.git",
        env: &[("LD_LIBRARY_PATH", "/lang/lib")],
    },
];
