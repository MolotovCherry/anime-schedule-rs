use const_format::formatcp;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    objects::{Categories, Category},
    Client, API_URL,
};

const API_CATEGORITES_TYPE: &str = formatcp!("{API_URL}/categories/{{categoryType}}");
const API_CATEGORITES_TYPE_SLUG: &str = formatcp!("{API_URL}/categories/{{categoryType}}/{{slug}}");

pub struct CategoryApi {
    client: Client,
    category_type: String,
}

impl CategoryApi {
    pub(crate) fn new(client: Client, category: &str) -> Self {
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
    client: Client,
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
    pub async fn send(self) -> Result<Categories, reqwest::Error> {
        let url = API_CATEGORITES_TYPE.replace("{categoryType}", &self.category_type);

        let query = serde_qs::to_string(&self).unwrap();

        let url = format!("{url}?{query}");

        let category: Categories = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.token.app_token())
            .send()
            .await?
            .json()
            .await?;

        Ok(category)
    }
}

/// Fetch the data of a specific category
pub struct CategorySlug {
    client: Client,
    slug: String,
    category_type: String,
}

impl CategorySlug {
    /// Fetch the data of a specific category
    pub async fn send(self) -> Result<Category, reqwest::Error> {
        let url = API_CATEGORITES_TYPE_SLUG
            .replace("{categoryType}", &self.category_type)
            .replace("{slug}", &self.slug);

        let category: Category = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.token.app_token())
            .send()
            .await?
            .json()
            .await?;

        Ok(category)
    }
}
