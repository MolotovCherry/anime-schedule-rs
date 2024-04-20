use http::HeaderMap;

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
