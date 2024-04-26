use std::{fmt, future::Future, pin::Pin, sync::Mutex, time::Duration};

use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AccessToken, AuthUrl, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RefreshToken, RevocationUrl,
    Scope, TokenResponse as _, TokenUrl,
};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct AppToken(String);

impl AppToken {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// Get the secret contained within this `AppToken`.
    ///
    /// # Security Warning
    ///
    /// Leaking this value may compromise the security of the OAuth2 flow.
    pub fn secret(&self) -> &String {
        &self.0
    }
}

impl std::fmt::Debug for AppToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AppToken([redacted])")
    }
}

pub type Callback = Box<
    dyn Fn(
            reqwest::Url,
            CsrfToken,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<(AuthorizationCode, CsrfToken), Box<dyn std::error::Error>>,
                    > + Send
                    + 'static,
            >,
        > + Send
        + 'static,
>;

/// A (de)serializable version of [Auth]. Only serializes the access token and its expiry.
/// This can be converted back to [Auth] if you provide your id, secret, app_token, and redirect url.
///
/// Callbacks are not saved or converted back. You must set it again manually.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    pub expires_at: u64,
}

impl AuthTokens {
    pub fn into_auth(
        self,
        client_id: ClientId,
        client_secret: ClientSecret,
        app_token: AppToken,
        redirect_url: RedirectUrl,
    ) -> Auth {
        let auth = Auth::new(client_id, client_secret, app_token, redirect_url);

        auth.set_access_token_unchecked(self.access_token);
        auth.set_refresh_token_unchecked(self.refresh_token);
        auth.set_expires_at_unchecked(self.expires_at);

        auth
    }
}

/// Manages oauth2 and client id, client secret, and app_token
///
/// Note that both access and refresh tokens are only valid for 3600 after issuance
pub struct Auth {
    client: BasicClient,
    app_token: AppToken,
    access_token: Mutex<AccessToken>,
    refresh_token: Mutex<RefreshToken>,
    // time in utc seconds when access and refresh token will expire
    // current api expiration is now + 3600
    expires_at: Mutex<u64>,
    scopes: Mutex<Vec<Scope>>,
    callback: tokio::sync::Mutex<Callback>,
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Auth { client, .. } = self;

        f.debug_struct("Auth")
            .field("client", &client)
            .field("app_token", &"[redacted]")
            .field("access_token", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .field("expires_at", &"[redacted]")
            .field("scopes", &"[redacted]")
            .field("callback", &"<ptr>")
            .finish()
    }
}

impl Auth {
    pub fn new(
        client_id: ClientId,
        client_secret: ClientSecret,
        app_token: AppToken,
        redirect_uri: RedirectUrl,
    ) -> Self {
        let client = BasicClient::new(
            client_id.clone(),
            Some(client_secret.clone()),
            AuthUrl::new(format!("{API_URL}/oauth2/authorize")).unwrap(),
            Some(TokenUrl::new(format!("{API_URL}/oauth2/token")).unwrap()),
        )
        .set_redirect_uri(redirect_uri.clone())
        .set_revocation_uri(RevocationUrl::new(format!("{API_URL}/oauth2/revoke")).unwrap());

        Self {
            client,
            app_token,
            access_token: Mutex::new(AccessToken::new(String::new())),
            refresh_token: Mutex::new(RefreshToken::new(String::new())),
            expires_at: Mutex::new(0),
            scopes: Mutex::new(Vec::new()),

            callback: tokio::sync::Mutex::new(Box::new(|_, _| {
                unimplemented!("oauth2 callback not implemented")
            })),
        }
    }

    /// Return client tokens to save user creds that can be serialized/deserialized.
    /// serializes access/refresh tokens, and their expiry
    /// Does not serialize client_id, client_secret, scopes, or callback
    pub fn to_tokens(&self) -> AuthTokens {
        let at = self.access_token();
        let rt = self.refresh_token();
        let ea = self.expires_at();

        AuthTokens {
            access_token: at,
            refresh_token: rt,
            expires_at: ea,
        }
    }

    /// Get the app token that was saved into this.
    pub fn app_token(&self) -> AppToken {
        self.app_token.clone()
    }

    /// Manually set the refresh token. This is handled automatically by [`Self::refresh()`], [`Self::refresh_blocking()`], [`Self::regenerate()`], and [`Self::regenerate_blocking()`].
    ///
    /// This method is safe in terms of no UB, however it is unchecked because it is possible to cause inconsistent state.
    ///
    /// Caller agrees to also set the correct access token expiry time as well.
    pub fn set_refresh_token_unchecked(&self, token: RefreshToken) {
        let mut lock = self.refresh_token.lock().unwrap();
        *lock = token;
    }

    /// Manually set the access token. This is handled automatically by [`Self::refresh()`], [`Self::refresh_blocking()`], [`Self::regenerate()`], and [`Self::regenerate_blocking()`].
    ///
    /// This method is safe in terms of no UB, however it is unchecked because it is possible to cause inconsistent state.
    ///
    /// Caller agrees to also set the correct access token expiry time as well.
    pub fn set_access_token_unchecked(&self, token: AccessToken) {
        let mut lock = self.access_token.lock().unwrap();
        *lock = token;
    }

    /// Updates the access token expiry time
    pub fn set_expires_in_unchecked(&self, duration: Duration) {
        let mut lock = self.expires_at.lock().unwrap();
        *lock = Utc::now().timestamp() as u64 + duration.as_secs();
    }

    /// Updates the access token expiry time
    pub fn set_expires_at_unchecked(&self, expiry: u64) {
        let mut lock = self.expires_at.lock().unwrap();
        *lock = expiry;
    }

    /// Add an oauth2 scope. Use this before you generate a new token.
    pub fn add_scope(&self, scope: Scope) {
        let mut lock = self.scopes.lock().unwrap();
        lock.push(scope);
    }

    /// Set the callback used when running [`Self::regenerate()`].
    /// This passes in a [`CsrfToken`] representing the client state this callback is looking for.
    /// You can know which client request is the correct client because the states match each other.
    ///
    /// You may return success from this function ONLY if the state is correct.
    /// You may want to make this timeout so [`Self::regenerate()`] doesn't block forever.
    pub async fn set_callback<
        F: Fn(reqwest::Url, CsrfToken) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(AuthorizationCode, CsrfToken), Box<dyn std::error::Error>>>
            + 'static
            + Send,
    >(
        &self,
        f: F,
    ) {
        let mut lock = self.callback.lock().await;
        *lock = Box::new(move |url, state| Box::pin(f(url, state)));
    }

    pub fn set_callback_blocking<
        F: Fn(reqwest::Url, CsrfToken) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(AuthorizationCode, CsrfToken), Box<dyn std::error::Error>>>
            + 'static
            + Send,
    >(
        &self,
        f: F,
    ) {
        RUNTIME.block_on(self.set_callback(f))
    }

    /// Is the access token valid?
    ///
    /// This checks that the access token's expiry is still valid.
    ///
    /// Unless you're doing manual setup, this will correctly represent whether it's valid or not
    ///
    /// (Manual setup is, for example, manually setting the access token)
    pub fn is_valid(&self) -> bool {
        (Utc::now().timestamp() as u64) < *self.expires_at.lock().unwrap()
    }

    /// Is the refresh token valid?
    ///
    /// This checks that the refresh token's expiry is still valid.
    ///
    /// Unless you're doing manual setup, this will correctly represent whether it's valid or not
    ///
    /// (Manual setup is, for example, manually setting the refresh token)
    pub fn is_refresh_valid(&self) -> bool {
        (Utc::now().timestamp() as u64) < *self.expires_at.lock().unwrap()
    }

    /// Revoke the access token
    pub async fn revoke_token(&self) -> Result<(), TokenError> {
        let token = self.access_token.lock().unwrap().clone();

        let req = self
            .client
            .revoke_token(oauth2::StandardRevocableToken::AccessToken(token))
            .map_err(|e| TokenError::Revoke(e.to_string()))?;

        req.request_async(async_http_client)
            .await
            .map_err(|e| TokenError::Revoke(e.to_string()))?;

        Ok(())
    }

    /// Revoke the access token
    pub fn revoke_token_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.revoke_token())
    }

    /// Revoke the refresh token
    pub async fn revoke_refresh_token(&self) -> Result<(), TokenError> {
        let token = self.refresh_token.lock().unwrap().clone();

        let req = self
            .client
            .revoke_token(oauth2::StandardRevocableToken::RefreshToken(token.clone()))
            .map_err(|e| TokenError::Revoke(e.to_string()))?;

        req.request_async(async_http_client)
            .await
            .map_err(|e| TokenError::Revoke(e.to_string()))?;

        Ok(())
    }

    /// Revoke the refresh token
    pub fn revoke_refresh_token_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.revoke_refresh_token())
    }

    /// Automatically regnerate token
    ///
    /// Does nothing if refresh token is not valid
    ///
    /// Note that both access and refresh tokens are only valid for 3600
    pub async fn try_refresh(&self) -> Result<(), TokenError> {
        // current access and refresh token expiry are the same: 3600

        if self.is_refresh_valid() {
            // try refresh token, if that fails we need to re-do it all
            self.refresh().await?;
        }

        Ok(())
    }

    /// Automatically regnerate token
    ///
    /// Does nothing if refresh token is not valid
    ///
    /// Note that both access and refresh tokens are only valid for 3600
    pub fn try_refresh_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.try_refresh())
    }

    /// Get access token
    pub fn access_token(&self) -> AccessToken {
        self.access_token.lock().unwrap().clone()
    }

    /// Get refresh token
    pub fn refresh_token(&self) -> RefreshToken {
        self.refresh_token.lock().unwrap().clone()
    }

    /// time in utc seconds when access and refresh token expires
    pub fn expires_at(&self) -> u64 {
        *self.expires_at.lock().unwrap()
    }

    /// exchange refresh token for new access token
    pub async fn refresh(&self) -> Result<(), TokenError> {
        let token = self.refresh_token.lock().unwrap().clone();

        let token = self
            .client
            .exchange_refresh_token(&token)
            .request_async(async_http_client)
            .await
            .map_err(|e| TokenError::OAuth2(e.to_string()))?;

        self.set_access_token_unchecked(token.access_token().clone());

        self.set_refresh_token_unchecked(token.refresh_token().unwrap().clone());

        self.set_expires_at_unchecked(
            (Utc::now().timestamp() as u64) + token.expires_in().unwrap().as_secs(),
        );

        Ok(())
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
        let (auth_code, client_state) = match callback(auth_url, state.clone()).await {
            Ok(v) => v,
            Err(e) => return Err(TokenError::Callback(e.to_string())),
        };

        // ensure state is correct
        if state.secret() != client_state.secret() {
            return Err(TokenError::StateMismatch);
        }

        // now get access token
        let Ok(token) = self
            .client
            .exchange_code(auth_code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
        else {
            return Err(TokenError::Access);
        };

        self.set_expires_at_unchecked(
            Utc::now().timestamp() as u64 + token.expires_in().unwrap().as_secs(),
        );

        self.set_access_token_unchecked(token.access_token().clone());

        self.set_refresh_token_unchecked(token.refresh_token().unwrap().clone());

        Ok(())
    }

    pub fn regenerate_blocking(&self) -> Result<(), TokenError> {
        RUNTIME.block_on(self.regenerate())
    }
}
