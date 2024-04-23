pub mod api;
mod api_request;
pub mod auth;
pub mod errors;
pub mod objects;
pub mod rate_limit;
mod utils;

use std::sync::Arc;

use reqwest::ClientBuilder;
use tokio::runtime::{Builder, Runtime};

use crate::{
    api::{
        account::AccountApi, anime::AnimeApi, animelists::AnimeListsApi, category::CategoryApi,
        timetables::TimetablesApi,
    },
    auth::{Auth, ClientError},
    utils::LazyLock,
};

use self::api_request::ApiRequest;

const API_URL: &str = "https://animeschedule.net/api/v3";

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
});

#[derive(Clone)]
pub struct AnimeScheduleClient {
    http: ApiRequest,
    pub auth: Arc<Auth>,
}

impl AnimeScheduleClient {
    /// Create client
    pub fn new(
        client_id: &str,
        client_secret: &str,
        app_token: &str,
        redirect_uri: &str,
    ) -> Result<Self, ClientError> {
        Self::new_with(
            client_id,
            client_secret,
            app_token,
            redirect_uri,
            |builder| {
                builder
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION"),
                    ))
                    .build()
            },
        )
    }

    /// Create client with custom reqwest settings (user agent for example)
    pub fn new_with(
        client_id: &str,
        client_secret: &str,
        app_token: &str,
        redirect_uri: &str,
        builder_cb: impl Fn(ClientBuilder) -> Result<reqwest::Client, reqwest::Error>,
    ) -> Result<Self, ClientError> {
        let builder = reqwest::Client::builder();
        let http = builder_cb(builder)?;

        let auth = Arc::new(Auth::new(
            client_id,
            client_secret,
            app_token,
            redirect_uri,
        )?);

        let http = ApiRequest::new(auth.clone(), http);

        let slf = Self { http, auth };

        Ok(slf)
    }

    /// Fetch anime data
    pub fn anime(&self) -> AnimeApi {
        AnimeApi::new(self.clone())
    }

    /// Fetch anime data
    pub fn animelists(&self) -> AnimeListsApi {
        AnimeListsApi::new(self.clone())
    }

    /// Fetch category data
    pub fn categories(&self, category: &str) -> CategoryApi {
        CategoryApi::new(self.clone(), category)
    }

    /// Fetch a week's timetable anime
    pub fn timetables(&self) -> TimetablesApi {
        TimetablesApi::new(self.clone())
    }

    /// Fetch account details
    pub fn account(&self) -> AccountApi {
        AccountApi::new(self.clone())
    }
}
