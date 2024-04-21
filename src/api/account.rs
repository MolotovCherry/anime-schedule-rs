use const_format::formatcp;
use reqwest::Url;

use crate::{
    errors::ApiError, objects::UserStats, rate_limit::RateLimit, AnimeScheduleClient, API_URL,
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

    pub async fn send(self) -> Result<(RateLimit, Url), ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::UserId);
        };

        let url = API_ACCOUNT_AVATAR.replace("{userId}", &user_id);

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.auth.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let is_err = response.status().is_server_error() || response.status().is_client_error();

        let text = response.text().await?;

        if is_err {
            return Err(ApiError::Api(text));
        }

        let url: Url = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), url))
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

    pub async fn send(self) -> Result<(RateLimit, Url), ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::UserId);
        };

        let url = API_ACCOUNT_BANNER.replace("{userId}", &user_id);

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.auth.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let is_err = response.status().is_server_error() || response.status().is_client_error();

        let text = response.text().await?;

        if is_err {
            return Err(ApiError::Api(text));
        }

        let url: Url = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), url))
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

    pub async fn send(self) -> Result<(RateLimit, UserStats), ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::UserId);
        };

        let url = API_ACCOUNT_STATS.replace("{userId}", &user_id);

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.auth.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let is_err = response.status().is_server_error() || response.status().is_client_error();

        let text = response.text().await?;

        if is_err {
            return Err(ApiError::Api(text));
        }

        let stats: UserStats = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), stats))
    }
}
