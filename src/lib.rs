pub mod api;
mod api_request;
pub mod auth;
pub mod errors;
pub mod objects;
pub mod rate_limit;
mod utils;

use std::sync::Arc;

pub use oauth2::{
    AccessToken, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RefreshToken,
    Scope,
};
use reqwest::{Client, ClientBuilder};
use tokio::runtime::{Builder, Runtime};

use crate::{
    api::{
        account::AccountApi, anime::AnimeApi, animelists::AnimeListsApi, category::CategoryApi,
        timetables::TimetablesApi,
    },
    auth::Auth,
    utils::LazyLock,
};

use self::{api_request::ApiRequest, errors::BuilderError};
pub use auth::AppToken;

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

/// A builder for [MalClient]
#[derive(Default)]
pub struct AnimeScheduleBuilder {
    auth: Option<Arc<Auth>>,
    client_id: Option<ClientId>,
    client_secret: Option<ClientSecret>,
    app_token: Option<AppToken>,
    redirect_url: Option<RedirectUrl>,
    #[allow(clippy::complexity)]
    http_cb: Option<Box<dyn FnOnce(ClientBuilder) -> Result<Client, reqwest::Error> + 'static>>,
}

impl AnimeScheduleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Use your own [Auth] value.
    ///
    /// If [Auth] is not provided, you must set client_id, client_secret, app_token, and redirect_url.
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(Arc::new(auth));
        self
    }

    /// Use a shared [Auth] value you have.
    ///
    /// If [Auth] is not provided, you must set client_id, client_secret, app_token, and redirect_url.
    pub fn auth_shared(mut self, auth: Arc<Auth>) -> Self {
        self.auth = Some(auth);
        self
    }

    /// The client id used to make a new [Auth]. No need to specify if you provided an [Auth] to the builder.
    pub fn client_id(mut self, client_id: ClientId) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// The client secret used to make a new [Auth]. No need to specify if you provided an [Auth] to the builder.
    pub fn client_secret(mut self, client_secret: ClientSecret) -> Self {
        self.client_secret = Some(client_secret);
        self
    }

    /// Your app token. No need to specify if you provided an [Auth] to the builder.
    pub fn app_token(mut self, app_token: AppToken) -> Self {
        self.app_token = Some(app_token);
        self
    }

    /// The redirect_url used to make a new [Auth]. No need to specify if you provided an [Auth] to the builder.
    pub fn redirect_url(mut self, redirect_url: RedirectUrl) -> Self {
        self.redirect_url = Some(redirect_url);
        self
    }

    /// Customize the reqwest client (e.g. change the useragent).
    pub fn http_builder(
        mut self,
        cb: impl FnOnce(ClientBuilder) -> Result<Client, reqwest::Error> + 'static,
    ) -> Self {
        self.http_cb = Some(Box::new(cb));
        self
    }

    pub fn build(self) -> Result<AnimeScheduleClient, BuilderError> {
        let auth = if let Some(auth) = self.auth {
            auth
        } else {
            let Some(client_id) = self.client_id else {
                return Err(BuilderError::Builder("client_id".to_owned()));
            };

            let Some(client_secret) = self.client_secret else {
                return Err(BuilderError::Builder("client_secret".to_owned()));
            };

            let Some(app_token) = self.app_token else {
                return Err(BuilderError::Builder("app_token".to_owned()));
            };

            let Some(redirect_url) = self.redirect_url else {
                return Err(BuilderError::Builder("redirect_url".to_owned()));
            };

            Arc::new(Auth::new(client_id, client_secret, app_token, redirect_url))
        };

        let http = if let Some(cb) = self.http_cb {
            let builder = ClientBuilder::new();
            cb(builder)?
        } else {
            ClientBuilder::new()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION"),
                ))
                .build()?
        };

        let http = ApiRequest::new(auth.clone(), http);

        let mal_client = AnimeScheduleClient { auth, http };

        Ok(mal_client)
    }
}
