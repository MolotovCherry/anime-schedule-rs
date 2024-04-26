use http::StatusCode;
use thiserror::Error;

use crate::auth::ClientError;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TokenError {
    #[error("failed to revoke token")]
    Revoke(String),
    #[error("callback failed")]
    Callback(String),
    #[error("refresh token is already expired")]
    Expired,
    #[error("{0}")]
    OAuth2(String),
    #[error("failed to refresh token")]
    Refresh,
    #[error("failed to generate access token")]
    Access,
    #[error("failed to parse uri: {0}")]
    Parse(#[from] ::oauth2::url::ParseError),
    #[error("state verification failed")]
    StateMismatch,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    ParseError(#[from] serde_json::Error),
    #[error("access token missing")]
    AccessTokenError,
    #[error("{status}: {error}")]
    ApiError { status: StatusCode, error: String },
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("api route requires etag")]
    Etag,
    #[error("api requires xml to be set")]
    Xml,
    #[error("api requires route")]
    Route,
    #[error("api requires user id")]
    UserId,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("field '{0}' is required")]
    Builder(String),
    #[error("{0}")]
    Client(#[from] ClientError),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}
