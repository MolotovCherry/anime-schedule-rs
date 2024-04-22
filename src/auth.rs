use std::{future::Future, pin::Pin, sync::Mutex, time::Duration};

use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AccessToken, AuthUrl, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken, RevocationUrl,
    Scope, TokenResponse as _, TokenUrl,
};

use crate::{errors::TokenError, API_URL, RUNTIME};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("failed to refresh token")]
    Refresh,
    #[error("{0:?}")]
    Token(#[from] TokenError),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct Code(pub String);
#[derive(Debug, Clone)]
pub struct State(pub String);

pub type Callback = Box<
    dyn Fn(
            reqwest::Url,
            State,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(Code, State), Box<dyn std::error::Error>>>
                    + Send
                    + 'static,
            >,
        > + Send
        + 'static,
>;

/// Note that both access and refresh tokens are only valid for 3600 after issuance
pub struct Auth {
    client: BasicClient,
    app_token: String,
    access_token: Mutex<Option<AccessToken>>,
    refresh_token: Mutex<Option<RefreshToken>>,
    // time in utc seconds when access and refresh token will expire
    // current api expiration is now + 3600
    expires_at: Mutex<Option<u64>>,
    scopes: Mutex<Vec<Scope>>,
    callback: tokio::sync::Mutex<Callback>,
}

impl Auth {
    pub(crate) fn new(
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

            callback: tokio::sync::Mutex::new(Box::new(|_, _| {
                unimplemented!("oauth2 callback not implemented")
            })),
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

    pub async fn set_callback<
        F: Fn(reqwest::Url, State) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(Code, State), Box<dyn std::error::Error>>> + 'static + Send,
    >(
        &self,
        f: F,
    ) {
        let mut lock = self.callback.lock().await;
        *lock = Box::new(move |url, state| Box::pin(f(url, state)));
    }

    pub fn set_callback_blocking<
        F: Fn(reqwest::Url, State) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(Code, State), Box<dyn std::error::Error>>> + 'static + Send,
    >(
        &self,
        f: F,
    ) {
        RUNTIME.block_on(self.set_callback(f))
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

    pub async fn revoke_token(&self) -> Result<(), TokenError> {
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

    pub fn revoke_token_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.revoke_token())
    }

    pub async fn revoke_refresh_token(&self) -> Result<(), TokenError> {
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

    pub fn revoke_refresh_token_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.revoke_refresh_token())
    }

    /// Automatically regnerate token if required
    /// Does nothing if there's no need to regenerate
    ///
    /// Note that both access and refresh tokens are only valid for 3600
    pub async fn try_refresh(&self) -> Result<(), TokenError> {
        // current access and refresh token expiry are the same: 3600

        let expires_at = *self.expires_at.lock().unwrap();
        if let Some(expires_at) = expires_at {
            let now = Utc::now().timestamp() as u64;

            // access token is expired, and refresh token exists, exchange refresh token
            if now >= expires_at
                || self.access_token.lock().unwrap().is_none()
                || self.refresh_token.lock().unwrap().is_none()
            {
                self.regenerate().await?;
            }
        } else if self.refresh_token().is_some() && self.access_token().is_none() {
            // try refresh token, if that fails we need to re-do it all
            if self.refresh().await.is_err() {
                self.regenerate().await?;
            }
        } else {
            // refresh token is None, no expire at time
            self.regenerate().await?;
        }

        Ok(())
    }

    pub fn try_refresh_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.try_refresh())
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

    pub fn refresh_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.refresh())
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
        let (res_code, res_state) = match callback(auth_url, State(state.secret().clone())).await {
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

    pub fn regenerate_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.regenerate())
    }
}
