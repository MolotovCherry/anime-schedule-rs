mod account;
mod anime;
mod lists;

use std::ops::{Deref, DerefMut};

pub use account::*;
pub use anime::*;
use chrono::{DateTime, FixedOffset};
pub use lists::*;
use serde::{Deserialize, Deserializer};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct Html(pub String);

impl Deref for Html {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Html {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// docs state that "0001-01-01T00:00:00Z" is a null value,
/// therefore this treats that value as None
fn datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    if s == "0001-01-01T00:00:00Z" {
        return Ok(None);
    }

    let datetime = DateTime::parse_from_rfc3339(s).map_err(serde::de::Error::custom)?;

    Ok(Some(datetime))
}
