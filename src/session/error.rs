//! Result type and errors related to session stuff.

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("wrong user credentials")]
    WrongCredentials,
    #[error("token is not of the correct kind")]
    WrongToken,
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    Http(#[from] crate::http::Error),
    #[error("{0}")]
    User(#[from] crate::user::error::Error),
    #[error("{0}")]
    Token(#[from] crate::token::error::Error),
    #[error("{0}")]
    Mfa(#[from] crate::mfa::error::Error),
    #[error("{0}")]
    Tonic(#[from] tonic::metadata::errors::InvalidMetadataValue),
    #[cfg(test)]
    #[error("unexpected error")]
    Debug,
}
