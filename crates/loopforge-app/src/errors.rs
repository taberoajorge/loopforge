use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not implemented")]
    NotImplemented,
}
