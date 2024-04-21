use const_format::formatcp;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    errors::ApiError,
    objects::{Categories, Category},
    rate_limit::RateLimit,
    utils::IsJson as _,
    AnimeScheduleClient, API_URL, RUNTIME,
};

const API_CATEGORITES_TYPE: &str = formatcp!("{API_URL}/categories/{{categoryType}}");
const API_CATEGORITES_TYPE_SLUG: &str = formatcp!("{API_URL}/categories/{{categoryType}}/{{slug}}");

pub struct CategoryApi {
    client: AnimeScheduleClient,
    category_type: String,
}

impl CategoryApi {
    pub(crate) fn new(client: AnimeScheduleClient, category: &str) -> Self {
        Self {
            client,
            category_type: category.to_owned(),
        }
    }

    pub fn get(&self) -> CategoryGet {
        CategoryGet {
            client: self.client.clone(),
            category_type: self.category_type.clone(),
            q: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CategoryGet {
    #[serde(skip)]
    client: AnimeScheduleClient,
    #[serde(skip)]
    category_type: String,

    /// Filter by text. Maximum length is 200.
    q: Option<String>,
}

impl CategoryGet {
    /// Fetch the data of a specific category
    pub fn slug(&self, slug: &str) -> CategorySlug {
        CategorySlug {
            client: self.client.clone(),
            slug: slug.to_owned(),
            category_type: self.category_type.clone(),
        }
    }

    /// Filter by text. Maximum length is 200.
    pub fn q(mut self, q: &str) -> Self {
        let mut q = q.to_owned();
        q.truncate(200);

        self.q = Some(q);
        self
    }

    /// Fetch the data of multiple categories by query
    pub async fn send(self) -> Result<(RateLimit, Categories), ApiError> {
        let url = API_CATEGORITES_TYPE.replace("{categoryType}", &self.category_type);

        let query = serde_qs::to_string(&self).unwrap();

        let url = format!("{url}?{query}");

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.auth.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let text = response.text().await?;

        if !text.is_json() {
            return Err(ApiError::Api(text));
        }

        let category: Categories = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), category))
    }

    pub fn send_blocking(self) -> Result<(RateLimit, Categories), ApiError> {
        RUNTIME.block_on(self.send())
    }
}

/// Fetch the data of a specific category
pub struct CategorySlug {
    client: AnimeScheduleClient,
    slug: String,
    category_type: String,
}

impl CategorySlug {
    /// Fetch the data of a specific category
    pub async fn send(self) -> Result<(RateLimit, Category), ApiError> {
        let url = API_CATEGORITES_TYPE_SLUG
            .replace("{categoryType}", &self.category_type)
            .replace("{slug}", &self.slug);

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.auth.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let text = response.text().await?;

        if !text.is_json() {
            return Err(ApiError::Api(text));
        }

        let category: Category = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), category))
    }

    pub fn send_blocking(self) -> Result<(RateLimit, Category), ApiError> {
        RUNTIME.block_on(self.send())
    }
}
