mod account;
mod anime;
mod lists;

use std::ops::{Deref, DerefMut};

pub use account::*;
pub use anime::*;
use axum::http::HeaderMap;
pub use lists::*;

/// The endpoints rate limit
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub struct RateLimit {
    /// the endpoint's limit
    pub limit: u16,
    /// how many requests you are allowed to make in the remaining time
    pub remaining: u16,
    /// a UNIX timestamp in seconds of when the rate limit resets
    pub reset: u64,
}

impl RateLimit {
    pub(crate) fn new(headers: &HeaderMap) -> Self {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .expect("x-ratelimit-remaining missing")
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        let reset = headers
            .get("x-ratelimit-reset")
            .expect("x-ratelimit-reset missing")
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        let limit = headers
            .get("x-ratelimit-limit")
            .expect("x-ratelimit-limit")
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        Self {
            limit,
            remaining,
            reset,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct Html(pub String);

impl Deref for Html {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Html {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
