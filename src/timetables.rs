use const_format::formatcp;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    objects::{AirTypeQuery, RateLimit, Timetables},
    Client, API_URL,
};

const API_TIMETABLES: &str = formatcp!("{API_URL}/timetables");
const API_TIMETABLES_AIR_TYPE: &str = formatcp!("{API_URL}/timetables/{{airType}}");

pub struct TimetablesApi {
    client: Client,
}

impl TimetablesApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn get(&self) -> TimetablesGet {
        TimetablesGet {
            client: self.client.clone(),
            air_type: None,
            week: None,
            year: None,
            tz: None,
        }
    }
}

/// Fetches an array of a week's timetable anime. Valid airType values are raw, sub, dub and all. Defaults to all.
#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TimetablesGet {
    #[serde(skip)]
    client: Client,
    #[serde(skip)]
    air_type: Option<AirTypeQuery>,

    /// The week's number in a year. Requires the year query parameter.
    week: Option<u16>,
    /// The year the requested week belongs in. Requires the week query parameter.
    year: Option<u16>,
    /// A IATA timezone string. Converts all of the times to that timezones. Defaults to Europe/London (GMT/BST.)
    /// Warning: It auto-converts for daylights savings if the target timezone has it.
    tz: Option<String>,
}

impl TimetablesGet {
    pub fn air_type(mut self, air_type: AirTypeQuery) -> Self {
        self.air_type = Some(air_type);
        self
    }

    /// The week's number in a year. Requires the year query parameter.
    pub fn week(mut self, week: u16) -> Self {
        self.week = Some(week);
        self
    }

    /// The year the requested week belongs in. Requires the week query parameter.
    pub fn year(mut self, year: u16) -> Self {
        self.year = Some(year);
        self
    }

    /// A IATA timezone string. Converts all of the times to that timezones. Defaults to Europe/London (GMT/BST.)
    /// Warning: It auto-converts for daylights savings if the target timezone has it.
    pub fn tz(mut self, tz: &str) -> Self {
        self.tz = Some(tz.to_owned());
        self
    }

    /// Fetch the data of multiple categories by query
    pub async fn send(self) -> Result<(RateLimit, Timetables), reqwest::Error> {
        let url = if let Some(air_type) = self.air_type {
            API_TIMETABLES_AIR_TYPE.replace("{airType}", air_type.into())
        } else {
            API_TIMETABLES.to_owned()
        };

        let query = serde_qs::to_string(&self).unwrap();

        let url = format!("{url}?{query}");

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.token.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let timetable: Timetables = response.json().await?;

        Ok((limit, timetable))
    }
}
