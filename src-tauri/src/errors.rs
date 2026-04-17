use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("api error ({status}): {msg}")]
    Api { status: u16, msg: String },

    #[error("config error: {0}")]
    Config(String),
}

pub type AppResult<T> = Result<T, AppError>;
