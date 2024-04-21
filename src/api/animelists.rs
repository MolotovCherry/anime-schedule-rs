use std::ops::Deref;

use chrono::prelude::*;
use const_format::formatcp;
use reqwest::multipart;

use crate::{
    errors::ApiError,
    objects::{Action, AutoScores, ListAnime, ListAnimePut, ListStatus, UserListAnime},
    rate_limit::RateLimit,
    utils::IsJson as _,
    AnimeScheduleClient, API_URL, RUNTIME,
};

const API_ANIMELISTS_USERID_ROUTE: &str = formatcp!("{API_URL}/animelists/{{userId}}/{{route}}");
const API_ANIMELISTS_ROUTE: &str = formatcp!("{API_URL}/animelists/oauth/{{route}}");
const API_ANIMELISTS_USERID: &str = formatcp!("{API_URL}/animelists/{{userId}}");
const API_ANIMELISTS: &str = formatcp!("{API_URL}/animelists/oauth");

pub struct AnimeListsApi {
    client: AnimeScheduleClient,
}

impl AnimeListsApi {
    pub(crate) fn new(client: AnimeScheduleClient) -> Self {
        Self { client }
    }

    /// Returns a specific List Anime object and an Etag in the response headers. Route is the anime's URL slug.
    pub fn get(&self) -> AnimeListsGet {
        AnimeListsGet {
            client: self.client.clone(),
            user_id: None,
        }
    }

    /// Import an anime list from MyAnimeList via .xml file
    pub fn put(&self) -> AnimeListsPut {
        AnimeListsPut {
            client: self.client.clone(),
            user_id: None,
            overwrite_mal_list: false,
            xml: None,
        }
    }

    /// Delete a user's specific List Anime
    pub fn delete(&self) -> AnimeListsDelete {
        AnimeListsDelete {
            client: self.client.clone(),
            route: None,
            user_id: None,
        }
    }
}

/// Returns a specific List Anime object and an Etag in the response headers. Route is the anime's URL slug.
pub struct AnimeListsGet {
    client: AnimeScheduleClient,

    /// user id to fetch from
    user_id: Option<String>,
}

impl AnimeListsGet {
    /// set the user id to get the lists from
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    /// set the route to get the lists from. Route is the anime's URL slug.
    pub fn route(self, route: &str) -> AnimeListsGetRoute {
        AnimeListsGetRoute {
            client: self.client.clone(),
            user_id: self.user_id,
            route: route.to_owned(),
        }
    }

    pub async fn send(self) -> Result<(RateLimit, UserListAnime), ApiError> {
        let is_user_id = self.user_id.is_some();

        let url = if let Some(user_id) = self.user_id {
            API_ANIMELISTS_USERID.replace("{userId}", &user_id)
        } else {
            API_ANIMELISTS.to_owned()
        };

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(if is_user_id {
                // access to another's list
                self.client.auth.app_token().to_owned()
            } else {
                // access to self list
                self.client
                    .auth
                    .access_token()
                    .ok_or(ApiError::AccessToken)?
                    .secret()
                    .clone()
            })
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let text = response.text().await?;

        if !text.is_json() {
            return Err(ApiError::Api(text));
        }

        let user_list: UserListAnime = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), user_list))
    }

    pub fn send_blocking(self) -> Result<(RateLimit, UserListAnime), ApiError> {
        RUNTIME.block_on(self.send())
    }
}

#[derive(Debug)]
pub struct ETag(pub String);
impl Deref for ETag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Returns a specific List Anime object and an Etag in the response headers. Route is the anime's URL slug.
pub struct AnimeListsGetRoute {
    client: AnimeScheduleClient,

    /// user id to fetch from
    user_id: Option<String>,
    /// route to fetch from
    route: String,
}

impl AnimeListsGetRoute {
    /// set the user id to get the lists from
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    pub async fn send(self) -> Result<(RateLimit, ETag, ListAnime), ApiError> {
        let is_user_id = self.user_id.is_some();

        let url = if let Some(user_id) = self.user_id {
            API_ANIMELISTS_USERID_ROUTE
                .replace("{userId}", &user_id)
                .replace("{route}", &self.route)
        } else {
            API_ANIMELISTS_ROUTE.replace("{route}", &self.route)
        };

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(if is_user_id {
                // access to another's list
                self.client.auth.app_token().to_owned()
            } else {
                // access to self list
                self.client
                    .auth
                    .access_token()
                    .ok_or(ApiError::AccessToken)?
                    .secret()
                    .clone()
            })
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);
        let etag = ETag(
            headers
                .get("etag")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default()
                .to_owned(),
        );

        let text = response.text().await?;

        if !text.is_json() {
            return Err(ApiError::Api(text));
        }

        let listanime: ListAnime = serde_json::from_str(&text)?;

        Ok((limit.unwrap(), etag, listanime))
    }

    pub fn send_blocking(self) -> Result<(RateLimit, ETag, ListAnime), ApiError> {
        RUNTIME.block_on(self.send())
    }
}

/// Import an anime list from MyAnimeList via .xml file
pub struct AnimeListsPut {
    client: AnimeScheduleClient,

    /// user id to put to
    user_id: Option<String>,
    /// whether to overwrite any preexisting List Anime with the ones being imported.
    overwrite_mal_list: bool,
    /// the myanimelist xml import file in the request. Up to 12mb in file size
    xml: Option<String>,
}

impl AnimeListsPut {
    pub fn route(self, route: &str) -> AnimeListsPutRoute {
        AnimeListsPutRoute {
            client: self.client,
            user_id: self.user_id,
            route: route.to_owned(),
            etag: None,
            list: ListAnimePut::default(),
        }
    }

    /// Set the user id to put to
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    /// whether to overwrite any preexisting List Anime with the ones being imported.
    /// default is false
    pub fn overwrite_mal_list(mut self, overwrite: bool) -> Self {
        self.overwrite_mal_list = overwrite;
        self
    }

    /// An MyAnimeList .xml import file in the request. Up to 12MB in file size.
    pub fn xml<S: Into<String>>(mut self, data: S) -> Self {
        let data = data.into();
        self.xml = Some(data);
        self
    }

    pub async fn send(self) -> Result<RateLimit, ApiError> {
        let url = if let Some(user_id) = self.user_id {
            API_ANIMELISTS_USERID.replace("{userId}", &user_id)
        } else {
            API_ANIMELISTS.to_owned()
        };

        let Some(xml) = self.xml else {
            return Err(ApiError::Xml);
        };

        // The docs do not say how to do this part
        // so this was reverse engineered from the site's xml importer
        // the site uses a different api url for this, but I'm still using
        // the officially listed api url
        //
        // reverse engineer from here:
        // https://animeschedule.net/users/<your_username>/settings/import-export
        let part = multipart::Part::bytes(xml.into_bytes())
            .file_name("list.xml")
            .mime_str("text/xml")
            .unwrap();

        let mut form = multipart::Form::new();
        if self.overwrite_mal_list {
            form = form.text("overwrite-mal-list", "on");
        }
        form = form.part("mal-list", part);

        let response = self
            .client
            .http
            .put(url)
            .bearer_auth(
                self.client
                    .auth
                    .access_token()
                    .ok_or(ApiError::AccessToken)?
                    .secret()
                    .clone(),
            )
            .multipart(form)
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let is_err = response.status().is_server_error() || response.status().is_client_error();

        let text = response.text().await?;

        if !text.is_empty() || is_err {
            return Err(ApiError::Api(text));
        }

        Ok(limit.unwrap())
    }

    pub fn send_blocking(self) -> Result<RateLimit, ApiError> {
        RUNTIME.block_on(self.send())
    }
}

/// Add/Update a specific List Anime for a user
pub struct AnimeListsPutRoute {
    client: AnimeScheduleClient,

    /// user id to put to
    user_id: Option<String>,
    /// the route's etag
    etag: Option<String>,
    /// route to put to
    route: String,
    /// the put list
    list: ListAnimePut,
}

impl AnimeListsPutRoute {
    /// Set the user id to put to
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    /// An Etag header in the request headers. Mandatory and must be valid. You can get a
    /// valid Etag by doing a GET request on a specific List Anime beforehand and getting it
    /// from the response headers.
    pub fn etag(mut self, etag: &str) -> Self {
        self.etag = Some(etag.to_owned());
        self
    }

    /// The list the anime belongs to.
    pub fn list_status(mut self, status: ListStatus) -> Self {
        self.list.list_status = Some(status);
        self
    }

    /// The amount of episodes seen from the anime.
    pub fn episodes_seen(mut self, seen: u64) -> Self {
        self.list.episodes_seen = Some(seen);
        self
    }

    /// The user's manually inputted score of the anime. From 0 to a 100.
    pub fn manual_score(mut self, score: u8) -> Self {
        self.list.manual_score = Some(score.clamp(0, 100));
        self
    }

    /// Whether to use automatic score calculation with multiple scores.
    pub fn use_auto_scores(mut self, use_auto_scores: bool) -> Self {
        self.list.use_auto_scores = Some(use_auto_scores);
        self
    }

    /// set auto scores
    pub fn auto_scores(mut self, scores: AutoScores) -> Self {
        self.list.auto_scores = Some(scores);
        self
    }

    /// The date the anime was started watching.
    pub fn start_date<Tz: TimeZone>(mut self, datetime: DateTime<Tz>) -> Self {
        let datetime = datetime.with_timezone(&Utc);
        self.list.start_date = Some(datetime);
        self
    }

    /// The date the anime was finished watching.
    pub fn end_date<Tz: TimeZone>(mut self, datetime: DateTime<Tz>) -> Self {
        let datetime = datetime.with_timezone(&Utc);
        self.list.end_date = Some(datetime);
        self
    }

    /// User note. Max length is 1000.
    pub fn note(mut self, note: &str) -> Self {
        let mut note = note.to_owned();
        note.truncate(1000);

        self.list.note = Some(note);
        self
    }

    /// Indicates a non-standard operation. Used only in PUT requests. Valid values are deleteNote.
    pub fn action(mut self, action: Action) -> Self {
        self.list.action = Some(action);
        self
    }

    pub async fn send(self) -> Result<RateLimit, ApiError> {
        if self.etag.is_none() {
            return Err(ApiError::Etag);
        }

        let url = if let Some(user_id) = self.user_id {
            API_ANIMELISTS_USERID_ROUTE
                .replace("{userId}", &user_id)
                .replace("{route}", &self.route)
        } else {
            API_ANIMELISTS_ROUTE.replace("{route}", &self.route)
        };

        let response = self
            .client
            .http
            .put(url)
            .header("ETag", self.etag.unwrap())
            .bearer_auth(
                self.client
                    .auth
                    .access_token()
                    .ok_or(ApiError::AccessToken)?
                    .secret()
                    .clone(),
            )
            .json(&self.list)
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let is_err = response.status().is_server_error() || response.status().is_client_error();

        let text = response.text().await?;

        if !text.is_empty() || is_err {
            return Err(ApiError::Api(text));
        }

        Ok(limit.unwrap())
    }

    pub fn send_blocking(self) -> Result<RateLimit, ApiError> {
        RUNTIME.block_on(self.send())
    }
}

/// Deletes a specific List Anime object from the user's anime list. Route is the anime's URL slug.
pub struct AnimeListsDelete {
    client: AnimeScheduleClient,

    /// anime url slug route to delete
    route: Option<String>,
    /// user id to delete from
    user_id: Option<String>,
}

impl AnimeListsDelete {
    /// set the user id to delete from
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_owned());
        self
    }

    /// set the route to delete from. this is mandatory
    pub fn route(mut self, route: &str) -> Self {
        self.route = Some(route.to_owned());
        self
    }

    pub async fn send(self) -> Result<RateLimit, ApiError> {
        let Some(route) = self.route else {
            return Err(ApiError::Route);
        };

        let url = if let Some(user_id) = self.user_id {
            API_ANIMELISTS_USERID_ROUTE
                .replace("{userId}", &user_id)
                .replace("{route}", &route)
        } else {
            API_ANIMELISTS_ROUTE.replace("{route}", &route)
        };

        let response = self
            .client
            .http
            .delete(url)
            .bearer_auth(
                self.client
                    .auth
                    .access_token()
                    .ok_or(ApiError::AccessToken)?
                    .secret()
                    .clone(),
            )
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let is_err = response.status().is_server_error() || response.status().is_client_error();

        let text = response.text().await?;

        if !text.is_empty() || is_err {
            return Err(ApiError::Api(text));
        }

        Ok(limit.unwrap())
    }

    pub fn send_blocking(self) -> Result<RateLimit, ApiError> {
        RUNTIME.block_on(self.send())
    }
}
