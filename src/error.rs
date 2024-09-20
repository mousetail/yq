#[derive(Debug)]
pub enum RunProcessError {
    NonZeroStatusCode(#[allow(unused)] Option<i32>),
    IOError(#[allow(unused)] std::io::Error),
}

impl From<std::io::Error> for RunProcessError {
    fn from(value: std::io::Error) -> Self {
        RunProcessError::IOError(value)
    }
}

#[derive(Debug)]
pub enum RunLangError {
    PluginInstallFailure(#[allow(unused)] RunProcessError),
    RunLangError(#[allow(unused)] RunProcessError),
    IOError(#[allow(unused)] std::io::Error),
}

impl From<std::io::Error> for RunLangError {
    fn from(value: std::io::Error) -> Self {
        RunLangError::IOError(value)
    }
}
