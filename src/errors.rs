#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
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
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Request(#[from] reqwest::Error),
    #[error("api returned: \"{0}\"")]
    Api(String),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("access token is missing")]
    AccessToken,
    #[error("api route requires etag")]
    Etag,
    #[error("api requires xml to be set")]
    Xml,
    #[error("api requires route")]
    Route,
}
