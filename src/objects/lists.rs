use chrono::prelude::*;

use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

#[derive(Deserialize, Clone)]
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
    pub start_date: DateTime<Utc>,
    /// The date the anime was finished watching.
    pub end_date: DateTime<Utc>,
    /// User note. Max length is 1000.
    pub note: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListAnimePut {
    /// The list the anime belongs to.
    pub list_status: Option<ListStatus>,
    /// The amount of episodes seen from the anime.
    pub episodes_seen: Option<u64>,
    /// The user's manually inputted score of the anime. From 0 to a 100.
    pub manual_score: Option<u8>,
    /// The user's automatically calculated average score of the anime. From 0 to a 100.
    pub average_auto_score: Option<u8>,
    /// Whether to use automatic score calculation with multiple scores.
    pub use_auto_scores: Option<bool>,
    pub auto_scores: Option<AutoScores>,
    /// The date the anime was started watching.
    pub start_date: Option<DateTime<Utc>>,
    /// The date the anime was finished watching.
    pub end_date: Option<DateTime<Utc>>,
    /// User note. Max length is 1000.
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, IntoStaticStr, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ListStatus {
    Completed,
    Watching,
    OnHold,
    Dropped,
    ToWatch,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoScores {
    pub score_one: AutoScore,
    pub score_two: AutoScore,
    pub score_three: AutoScore,
    pub score_four: AutoScore,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoScore {
    /// The score's text/meaning.
    pub score_text: String,
    /// The score's numerical value. From 0 to a 100.
    pub score: u8,
}

#[derive(Deserialize, Clone)]
pub struct CustomList {
    /// The name of the custom list.
    pub name: String,
    /// The URL slug of the custom list.
    pub route: String,
}
