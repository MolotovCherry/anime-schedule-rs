use std::sync::Arc;

use http::HeaderMap;
use reqwest::{Client, IntoUrl, RequestBuilder};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{errors::ApiError, rate_limit::RateLimit, utils::IsJson, Auth};

#[derive(Copy, Clone, Debug)]
pub(crate) enum RequestMethod {
    Get,
    Put,
    Delete,
}

pub(crate) struct ApiRequest {
    // these fields are synced between all clients
    auth: Arc<Auth>,
    http: reqwest::Client,
    // these are not
    #[allow(clippy::complexity)]
    response_cb: Option<Box<dyn FnOnce(&HeaderMap) + 'static>>,
    request_cb: Option<Box<dyn FnOnce(RequestBuilder) -> RequestBuilder + 'static>>,
}

impl Clone for ApiRequest {
    fn clone(&self) -> Self {
        let ApiRequest { auth, http, .. } = self;

        ApiRequest {
            auth: auth.clone(),
            http: http.clone(),
            // we don't need to clone this. it's set individually per call, and runs only once
            response_cb: None,
            request_cb: None,
        }
    }
}

impl ApiRequest {
    pub fn new(auth: Arc<Auth>, http: Client) -> Self {
        Self {
            auth,
            http,
            response_cb: None,
            request_cb: None,
        }
    }

    pub fn response_cb(&mut self, response_cb: impl FnOnce(&HeaderMap) + 'static) {
        self.response_cb = Some(Box::new(response_cb));
    }

    pub fn request_cb(
        &mut self,
        request_cb: impl FnOnce(RequestBuilder) -> RequestBuilder + 'static,
    ) {
        self.request_cb = Some(Box::new(request_cb));
    }

    pub async fn get<D>(
        &mut self,
        url: impl IntoUrl,
        is_auth: bool,
    ) -> Result<(RateLimit, D), ApiError>
    where
        D: DeserializeOwned,
    {
        self.api_request(url.into_url()?, RequestMethod::Get, is_auth)
            .await
    }

    pub async fn delete<D>(
        &mut self,
        url: impl IntoUrl,
        is_auth: bool,
    ) -> Result<(RateLimit, D), ApiError>
    where
        D: DeserializeOwned,
    {
        self.api_request(url.into_url()?, RequestMethod::Delete, is_auth)
            .await
    }

    pub async fn put<D>(
        &mut self,
        url: impl IntoUrl,
        is_auth: bool,
    ) -> Result<(RateLimit, D), ApiError>
    where
        D: DeserializeOwned,
    {
        self.api_request(url.into_url()?, RequestMethod::Put, is_auth)
            .await
    }

    /// is_auth : Use user authentication in request; otherwise use ClientID header
    async fn api_request<D>(
        &mut self,
        url: impl IntoUrl,
        method: RequestMethod,
        // whether to use oauth2 access token or client id header
        is_auth: bool,
    ) -> Result<(RateLimit, D), ApiError>
    where
        D: DeserializeOwned,
    {
        let request = match method {
            RequestMethod::Get => self.http.get(url.into_url()?),
            RequestMethod::Delete => self.http.delete(url.into_url()?),
            RequestMethod::Put => self.http.put(url.into_url()?),
        };

        let request = if is_auth {
            request.bearer_auth(
                self.auth
                    .access_token()
                    .ok_or(ApiError::AccessTokenError)?
                    .secret(),
            )
        } else {
            request.bearer_auth(self.auth.app_token().secret())
        };

        let request = if let Some(cb) = self.request_cb.take() {
            cb(request)
        } else {
            request
        };

        let response = request.send().await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        if let Some(cb) = self.response_cb.take() {
            cb(headers);
        }

        let status = response.status();
        let text = response.text().await?;

        debug!(status = status.as_u16(), response = text);

        if !text.is_json() {
            return Err(ApiError::ApiError {
                status,
                error: text,
            });
        }

        let data = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), data))
    }
}
