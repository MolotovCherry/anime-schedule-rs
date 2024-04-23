use const_format::formatcp;
use reqwest::Url;

use crate::{
    errors::ApiError, objects::UserStats, rate_limit::RateLimit, AnimeScheduleClient, API_URL,
    RUNTIME,
};

const API_ACCOUNT_AVATAR: &str = formatcp!("{API_URL}/users/{{userId}}/avatar");
const API_ACCOUNT_BANNER: &str = formatcp!("{API_URL}/users/{{userId}}/banner");
const API_ACCOUNT_STATS: &str = formatcp!("{API_URL}/users/{{userId}}/stats");

pub struct AccountApi {
    client: AnimeScheduleClient,
}

impl AccountApi {
    pub(crate) fn new(client: AnimeScheduleClient) -> Self {
        Self { client }
    }

    pub fn get(&self) -> AccountApiGet {
        AccountApiGet {
            client: self.client.clone(),
        }
    }
}

pub struct AccountApiGet {
    client: AnimeScheduleClient,
}

impl AccountApiGet {
    /// Fetch a user's profile avatar URL
    pub fn avatar(self) -> AccountApiAvatar {
        AccountApiAvatar {
            client: self.client,
            user_id: None,
        }
    }

    /// Fetch a user's profile banner URL
    pub fn banner(self) -> AccountApiBanner {
        AccountApiBanner {
            client: self.client,
            user_id: None,
        }
    }

    /// Fetch a user's stats
    pub fn stats(self) -> AccountApiStats {
        AccountApiStats {
            client: self.client,
            user_id: None,
        }
    }
}

pub struct AccountApiAvatar {
    client: AnimeScheduleClient,
    user_id: Option<String>,
}

impl AccountApiAvatar {
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    pub async fn send(mut self) -> Result<(RateLimit, Url), ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::UserId);
        };

        let url = API_ACCOUNT_AVATAR.replace("{userId}", &user_id);

        self.client.http.get(url, false).await
    }

    pub fn send_blocking(self) -> Result<(RateLimit, Url), ApiError> {
        RUNTIME.block_on(self.send())
    }
}

pub struct AccountApiBanner {
    client: AnimeScheduleClient,
    user_id: Option<String>,
}

impl AccountApiBanner {
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    pub async fn send(mut self) -> Result<(RateLimit, Url), ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::UserId);
        };

        let url = API_ACCOUNT_BANNER.replace("{userId}", &user_id);

        self.client.http.get(url, false).await
    }

    pub fn send_blocking(self) -> Result<(RateLimit, Url), ApiError> {
        RUNTIME.block_on(self.send())
    }
}

pub struct AccountApiStats {
    client: AnimeScheduleClient,
    user_id: Option<String>,
}

impl AccountApiStats {
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    pub async fn send(mut self) -> Result<(RateLimit, UserStats), ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::UserId);
        };

        let url = API_ACCOUNT_STATS.replace("{userId}", &user_id);

        self.client.http.get(url, false).await
    }

    pub fn send_blocking(self) -> Result<(RateLimit, UserStats), ApiError> {
        RUNTIME.block_on(self.send())
    }
}
