pub mod api;
pub mod errors;
pub mod objects;
pub mod rate_limit;

mod utils;

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use self::api::account::AccountApi;
use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AccessToken, AuthUrl, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken, RevocationUrl,
    Scope, TokenResponse as _, TokenUrl,
};
use reqwest::ClientBuilder;
#[cfg(feature = "callback_server")]
use {
    axum::{extract::Query, response::Html},
    serde::Deserialize,
    tokio::{sync::oneshot, task},
};

use crate::{
    api::{
        anime::AnimeApi, animelists::AnimeListsApi, category::CategoryApi,
        timetables::TimetablesApi,
    },
    errors::TokenError,
};

const API_URL: &str = "https://animeschedule.net/api/v3";

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    token: Arc<Token>,
}

impl Client {
    /// Create client
    pub fn new(
        client_id: &str,
        client_secret: &str,
        app_token: &str,
        redirect_uri: &str,
    ) -> Result<Self, ClientError> {
        let http = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            ))
            .build()
            .unwrap();
        let token = Token::new(client_id, client_secret, app_token, redirect_uri)?;

        let slf = Self {
            http,
            token: Arc::new(token),
        };

        Ok(slf)
    }

    /// Create client with custom reqwest settings (user agent for example)
    pub fn new_with_ua(
        client_id: &str,
        client_secret: &str,
        app_token: &str,
        redirect_uri: &str,
        builder_cb: impl Fn(&mut ClientBuilder),
    ) -> Result<Self, ClientError> {
        let mut builder = reqwest::Client::builder();
        builder_cb(&mut builder);
        let http = builder.build().unwrap();

        let token = Token::new(client_id, client_secret, app_token, redirect_uri)?;

        let slf = Self {
            http,
            token: Arc::new(token),
        };

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

    pub fn token(&self) -> &Token {
        &self.token
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("failed to refresh token")]
    Refresh,
    #[error("{0:?}")]
    Token(#[from] TokenError),
}

#[derive(Debug)]
pub struct Code(pub String);
#[derive(Debug)]
pub struct State(pub String);

pub type Callback = Box<
    dyn Fn(
            reqwest::Url,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(Code, State), Box<dyn std::error::Error>>>
                    + Send
                    + 'static,
            >,
        > + Send
        + 'static,
>;

pub struct Token {
    client: BasicClient,
    app_token: String,
    access_token: Mutex<Option<AccessToken>>,
    refresh_token: Mutex<Option<RefreshToken>>,
    // time in utc seconds when access token will expire
    expires_at: Mutex<Option<u64>>,
    scopes: Mutex<Vec<Scope>>,
    callback: tokio::sync::Mutex<Callback>,
}

impl Token {
    fn new(
        client_id: &str,
        client_secret: &str,
        app_token: &str,
        redirect_uri: &str,
    ) -> Result<Self, TokenError> {
        let client = BasicClient::new(
            ClientId::new(client_id.to_owned()),
            Some(ClientSecret::new(client_secret.to_owned())),
            AuthUrl::new(format!("{API_URL}/oauth2/authorize")).unwrap(),
            Some(TokenUrl::new(format!("{API_URL}/oauth2/token")).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri.to_owned())?)
        .set_revocation_uri(RevocationUrl::new(format!("{API_URL}/oauth2/revoke")).unwrap());

        let slf = Self {
            client,
            app_token: app_token.to_owned(),
            access_token: Mutex::new(None),
            refresh_token: Mutex::new(None),
            expires_at: Mutex::new(None),
            scopes: Mutex::new(Vec::new()),

            callback: tokio::sync::Mutex::new(
                #[cfg(feature = "callback_server")]
                Self::make_callback("127.0.0.1", 8888),
                #[cfg(not(feature = "callback_server"))]
                Box::new(|_| unimplemented!("oauth2 callback not implemented")),
            ),
        };

        Ok(slf)
    }

    pub fn app_token(&self) -> &str {
        &self.app_token
    }

    pub fn set_refresh_token(&self, token: Option<&str>) {
        let mut lock = self.refresh_token.lock().unwrap();
        *lock = token.map(|t| RefreshToken::new(t.to_owned()));
    }

    pub fn set_access_token(&self, token: &str) {
        let mut lock = self.access_token.lock().unwrap();
        *lock = Some(AccessToken::new(token.to_owned()));
    }

    /// Updates the access token expiry time
    pub fn set_expires_in(&self, duration: Option<Duration>) {
        let mut lock = self.expires_at.lock().unwrap();
        *lock = duration.map(|d| Utc::now().timestamp() as u64 + d.as_secs());
    }

    pub fn add_scope(&self, scope: &str) {
        let mut lock = self.scopes.lock().unwrap();
        lock.push(Scope::new(scope.to_owned()));
    }

    /// set the uri / port callback server listens on
    #[cfg(feature = "callback_server")]
    pub async fn set_callback_server(&self, host: &str, port: u16) {
        let mut lock = self.callback.lock().await;
        *lock = Self::make_callback(host, port);
    }

    /// Open webbrowser with url
    #[cfg(feature = "callback_server")]
    fn make_callback(host: &str, port: u16) -> Callback {
        let host = host.to_owned();

        Box::new(move |url| {
            let host = host.clone();

            Box::pin(async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel::<(Code, State)>(1);
                let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

                task::spawn(async move {
                    #[derive(Debug, Deserialize)]
                    struct Params {
                        code: String,
                        state: String,
                    }

                    let app = axum::Router::new().route(
                        "/",
                        axum::routing::get(|Query(params): Query<Params>| async move {
                            _ = tx.send((Code(params.code), State(params.state))).await;
                            Html("<h1>Token saved</h1>")
                        }),
                    );

                    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
                        .await
                        .unwrap();

                    axum::serve(listener, app)
                        .with_graceful_shutdown(async {
                            shutdown_rx.await.ok();
                        })
                        .await
                        .unwrap();
                });

                webbrowser::open(url.as_str())?;

                let Some((code, state)) = rx.recv().await else {
                    return Err("channel unexpectedly closed".into());
                };

                _ = shutdown_tx.send(());

                Ok((code, state))
            })
        })
    }

    pub async fn set_callback<
        F: Fn(reqwest::Url) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(Code, State), Box<dyn std::error::Error>>> + 'static + Send,
    >(
        &mut self,
        f: F,
    ) {
        let mut lock = self.callback.lock().await;
        *lock = Box::new(move |url| Box::pin(f(url)));
    }

    /// Is the current state of this token valid?
    ///
    /// This checks that the access token exists and its expiry is still valid,
    /// that a refresh token also exists.
    ///
    /// Unless you're doing manual setup, this will correctly represent whether it's valid or not
    ///
    /// (Manual setup is, for example, manually setting the refresh token and running refresh on it)
    pub fn is_valid(&self) -> bool {
        let has_access_token = self.access_token.lock().unwrap().is_some();
        let has_refresh_token = self.refresh_token.lock().unwrap().is_some();
        let is_active = self
            .expires_at
            .lock()
            .unwrap()
            .is_some_and(|t| (Utc::now().timestamp() as u64) < t);

        has_access_token && has_refresh_token && is_active
    }

    pub async fn revoke_token(&mut self) -> Result<(), TokenError> {
        let token = self.access_token.lock().unwrap().clone();
        if let Some(token) = token {
            let req = self
                .client
                .revoke_token(oauth2::StandardRevocableToken::AccessToken(token))
                .map_err(|e| TokenError::Revoke(e.to_string()))?;

            req.request_async(async_http_client)
                .await
                .map_err(|e| TokenError::Revoke(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn revoke_refresh_token(&mut self) -> Result<(), TokenError> {
        let token = self.refresh_token.lock().unwrap().clone();
        if let Some(token) = token {
            let req = self
                .client
                .revoke_token(oauth2::StandardRevocableToken::RefreshToken(token.clone()))
                .map_err(|e| TokenError::Revoke(e.to_string()))?;

            req.request_async(async_http_client)
                .await
                .map_err(|e| TokenError::Revoke(e.to_string()))?;
        }

        Ok(())
    }

    /// Automatically regnerate token if required
    /// Does nothing if there's no need to regenerate
    pub async fn try_refresh(&self) -> Result<(), TokenError> {
        let expires_at = *self.expires_at.lock().unwrap();
        if let Some(expires_at) = expires_at {
            let now = Utc::now().timestamp() as u64;

            // access token is expired, and refresh token exists, exchange refresh token
            if now >= expires_at && self.refresh_token.lock().unwrap().is_some() {
                if self.refresh().await.is_err() {
                    self.regenerate().await?;
                }
            }
            // access token is empty, regenerate whole thing
            else if self.access_token.lock().unwrap().is_none()
                || self.refresh_token.lock().unwrap().is_none()
            {
                self.regenerate().await?;
            }
        } else if self.refresh_token().is_some() {
            // no expiry, but refresh token is set
            if self.refresh().await.is_err() {
                self.regenerate().await?;
            }
        } else {
            // refresh token is None, no expire at time
            self.regenerate().await?;
        }

        Ok(())
    }

    /// get access token and expiry time in utc
    pub fn access_token(&self) -> Option<AccessToken> {
        self.access_token.lock().unwrap().clone()
    }

    /// get refresh token
    pub fn refresh_token(&self) -> Option<RefreshToken> {
        self.refresh_token.lock().unwrap().clone()
    }

    /// time in utc seconds when access token expires
    pub fn expires_at(&self) -> Option<u64> {
        *self.expires_at.lock().unwrap()
    }

    /// exchange refresh token for new access token
    pub async fn refresh(&self) -> Result<(), TokenError> {
        let token = self.refresh_token.lock().unwrap().clone();
        if let Some(refresh_token) = token {
            let token = self
                .client
                .exchange_refresh_token(&refresh_token)
                .request_async(async_http_client)
                .await
                .map_err(|e| TokenError::OAuth2(e.to_string()))?;

            let mut lock = self.access_token.lock().unwrap();
            *lock = Some(token.access_token().clone());

            let mut lock = self.refresh_token.lock().unwrap();
            *lock = token.refresh_token().cloned();

            let mut lock = self.expires_at.lock().unwrap();
            *lock = token
                .expires_in()
                .map(|d| (Utc::now().timestamp() as u64) + d.as_secs());

            Ok(())
        } else {
            Err(TokenError::Refresh)
        }
    }

    /// regenerate fresh access and refresh tokens
    pub async fn regenerate(&self) -> Result<(), TokenError> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let scopes = self.scopes.lock().unwrap().clone();

        let (auth_url, state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(scopes.into_iter())
            .set_pkce_challenge(pkce_challenge)
            .url();

        let callback = self.callback.lock().await;
        let (res_code, res_state) = match callback(auth_url).await {
            Ok(v) => v,
            Err(e) => return Err(TokenError::Callback(e.to_string())),
        };

        // ensure state is correct
        if state.secret() != &res_state.0 {
            return Err(TokenError::StateMismatch);
        }

        // now get access token
        let Ok(token) = self
            .client
            .exchange_code(AuthorizationCode::new(res_code.0))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
        else {
            return Err(TokenError::Access);
        };

        let mut expires_at = self.expires_at.lock().unwrap();
        *expires_at = token
            .expires_in()
            .map(|d| Utc::now().timestamp() as u64 + d.as_secs());

        let mut access_token = self.access_token.lock().unwrap();
        *access_token = Some(token.access_token().clone());

        let mut refresh_token = self.refresh_token.lock().unwrap();
        *refresh_token = token.refresh_token().cloned();

        Ok(())
    }
}
