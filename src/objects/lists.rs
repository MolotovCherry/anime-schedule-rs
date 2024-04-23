use std::{collections::HashMap, ops::Deref};

use chrono::prelude::*;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::IntoStaticStr;

use super::datetime_opt;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Route(pub String);
impl Deref for Route {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserListAnime {
    pub user_id: String,
    pub shows: HashMap<Route, ListAnime>,
    pub custom_lists: Option<Vec<CustomList>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListAnime {
    /// The unique URL slug of the anime.
    pub route: String,
    /// The list the anime belongs to.
    pub list_status: ListStatus,
    /// The amount of episodes seen from the anime.
    pub episodes_seen: u64,
    /// The user's manually inputted score of the anime. From 0 to a 100.
    pub manual_score: Option<u8>,
    /// The user's automatically calculated average score of the anime. From 0 to a 100.
    pub average_auto_score: Option<u8>,
    /// Whether to use automatic score calculation with multiple scores.
    pub use_auto_scores: bool,
    pub auto_scores: AutoScores,
    /// The date the anime was started watching.
    #[serde(default, deserialize_with = "datetime_opt")]
    pub start_date: Option<DateTime<FixedOffset>>,
    /// The date the anime was finished watching.
    #[serde(default, deserialize_with = "datetime_opt")]
    pub end_date: Option<DateTime<FixedOffset>>,
    /// User note. Max length is 1000.
    pub note: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListAnimePut {
    /// The list the anime belongs to.
    pub list_status: Option<ListStatus>,
    /// The amount of episodes seen from the anime.
    pub episodes_seen: Option<u64>,
    /// The user's manually inputted score of the anime. From 0 to a 100.
    pub manual_score: Option<u8>,
    /// Whether to use automatic score calculation with multiple scores.
    pub use_auto_scores: Option<bool>,
    pub auto_scores: Option<AutoScores>,
    /// The date the anime was started watching.
    pub start_date: Option<DateTime<FixedOffset>>,
    /// The date the anime was finished watching.
    pub end_date: Option<DateTime<FixedOffset>>,
    /// User note. Max length is 1000.
    pub note: Option<String>,
    /// Indicates a non-standard operation. Used only in PUT requests. Valid values are deleteNote.
    pub action: Option<Action>,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    DeleteNode,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, IntoStaticStr, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ListStatus {
    Completed,
    Watching,
    OnHold,
    Dropped,
    ToWatch,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoScores {
    pub score_one: AutoScore,
    pub score_two: AutoScore,
    pub score_three: AutoScore,
    pub score_four: AutoScore,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoScore {
    /// The score's text/meaning.
    pub score_text: String,
    /// The score's numerical value. From 0 to a 100.
    pub score: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomList {
    /// The name of the custom list.
    pub name: String,
    /// The URL slug of the custom list.
    pub route: String,
}
