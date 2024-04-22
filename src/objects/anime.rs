use std::ops::{Deref, DerefMut};

use chrono::prelude::*;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use strum::IntoStaticStr;

use super::Html;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnimePage {
    pub page: u64,
    pub total_amount: u64,
    pub anime: Vec<Anime>,
}

/// Anime object to be used with the Anime endpoint
/// https://animeschedule.net/api/v3/documentation/anime
///
/// japan datetimes are in japan fixed offset timezone. the rest are utc
/// (you deal with utc ones according to actual date)
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    /// The unique ID.
    pub id: String,
    /// The title. Separate from other names and used as a high-priority name in some cases.
    pub title: String,
    /// The unique URL slug.
    pub route: String,
    /// The Japanese release date of the first episode.
    #[serde(default, deserialize_with = "jpn_datetime_opt")]
    pub premier: Option<DateTime<FixedOffset>>,
    /// The English Sub release date of the first episode.
    pub sub_premier: Option<DateTime<Utc>>,
    /// The English Dub release date of the first episode.
    pub dub_premier: Option<DateTime<Utc>>,
    /// The earliest month of an anime's release date.
    pub month: Option<Month>,
    /// The earliest year of an anime's release date.
    pub year: Option<u64>,
    pub season: Season,
    /// The delayed text on the timetable.
    pub delayed_timetable: Option<DelayedTimetable>,
    /// The date from which it has been delayed.
    pub delayed_from: DateTime<Utc>,
    /// The date until it has been delayed to.
    pub delayed_until: DateTime<Utc>,
    /// The sub delayed text on the timetable. Used only if SubPremier is not null.
    pub sub_delayed_timetable: Option<DateTime<Utc>>,
    /// The date from which the sub has been delayed. Used only if SubPremier is not null.
    pub sub_delayed_from: Option<DateTime<Utc>>,
    /// The date until it the sub has been delayed to. Used only if SubPremier is not null.
    pub sub_delayed_until: Option<DateTime<Utc>>,
    /// The dub delayed text on the timetable. Used only if DubPremier is not null.
    pub dub_delayed_timetable: Option<DelayedTimetable>,
    /// The date from which the dub has been delayed from. Used only if DubPremier is not null.
    pub dub_delayed_from: Option<DateTime<Utc>>,
    /// The date until it the dub has been delayed to. Used only if DubPremier is not null.
    pub dub_delayed_until: Option<DateTime<Utc>>,
    /// The delayed description text on the anime's page.
    pub delayed_desc: Option<String>,
    /// The Japanese release time. Only the hour and minute are relevant.
    #[serde(deserialize_with = "jpn_datetime")]
    pub jpn_time: DateTime<FixedOffset>,
    /// The English Sub release time. Only the hour and minute are relevant.
    pub sub_time: DateTime<Utc>,
    /// The English Dub release time. Only the hour and minute are relevant.
    pub dub_time: DateTime<Utc>,
    /// The description.
    pub description: Html,
    /// The anime's genres in an array of the category object.
    pub genres: Vec<Category>,
    /// The anime's studios in an array of the category object.
    pub studios: Vec<Category>,
    /// The anime's sources in an array of the category object.
    pub sources: Vec<Category>,
    /// The anime's media types in an array of the category object.
    pub media_types: Vec<Category>,
    /// The number of episodes.
    pub episodes: Option<u64>,
    /// The length per episode in minutes.
    pub length_min: Option<u64>,
    /// The airing status.
    pub status: AirStatus,
    /// The anime's poster/image URL slug.
    pub image_version_route: String,
    pub stats: Stats,
    pub days: Option<Days>,
    pub names: Option<Names>,
    pub relations: Option<Relations>,
    pub websites: Websites,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub title: String,
    /// The year the season is in.
    pub year: String,
    /// The calendar season.
    pub season: String,
    /// The unique URL slug, consisting of the calendar season and the year.
    pub route: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct Categories(pub Vec<Category>);
impl Deref for Categories {
    type Target = Vec<Category>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Categories {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Category {
    pub name: String,
    /// The unique URL slug.
    pub route: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    /// The average score from 1 to a 100. The score is weighted with the formula
    /// ratingCount/(ratingCount+5)*ratingSum/ratingCount+(5/(ratingCount+5))*mean.
    /// Mean is the average score across all anime.
    pub average_score: f64,
    /// How many users have rated/scored.
    pub rating_count: u64,
    /// How many users have it in their anime list.
    pub tracked_count: u64,
    /// Popularity rating compared to all other anime.
    pub tracked_rating: u64,
    /// The HEX color value for Average Score's color in default theme mode.
    pub color_light_mode: String,
    /// The HEX color value for Average Score's color in dark theme mode.
    pub color_dark_mode: String,
}

/// Anime airing status
#[derive(Deserialize, Clone, IntoStaticStr, Debug, PartialEq)]
pub enum AirStatus {
    Upcoming,
    Ongoing,
    Delayed,
    Finished,
}

/// Anime airing status
#[derive(Serialize, Clone, IntoStaticStr, Debug, PartialEq)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum AirStatusQuery {
    Upcoming,
    Ongoing,
    Finished,
}

/// Whether an anime airs multiple times a week and which days specifically.
/// Used only if it airs multiple times a week.
#[derive(Deserialize, Clone, Debug)]
pub struct Days {
    pub sunday: bool,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Names {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub abbreviation: Option<String>,
    pub synonyms: Option<Vec<String>>,
}

/// All related anime. The strings represent their route/slug.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Relations {
    pub sequels: Option<Vec<String>>,
    pub prequels: Option<Vec<String>>,
    pub parents: Option<Vec<String>>,
    pub alternatives: Option<Vec<String>>,
    pub other: Option<Vec<String>>,
    pub side_stories: Option<Vec<String>>,
    pub spinoffs: Option<Vec<String>>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Websites {
    pub official: Option<String>,
    pub mal: Option<String>,
    pub ani_list: Option<String>,
    pub kitsu: Option<String>,
    pub anime_planet: Option<String>,
    pub anidb: Option<String>,
    pub crunchyroll: Option<String>,
    pub funimation: Option<String>,
    pub wakanim: Option<String>,
    pub amazon: Option<String>,
    pub hidive: Option<String>,
    pub hulu: Option<String>,
    pub youtube: Option<String>,
    pub netflix: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Streams {
    pub crunchyroll: Option<String>,
    pub funimation: Option<String>,
    pub wakanim: Option<String>,
    pub amazon: Option<String>,
    pub hidive: Option<String>,
    pub hulu: Option<String>,
    pub youtube: Option<String>,
    pub netflix: Option<String>,
}

#[derive(Serialize, Clone, IntoStaticStr, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StreamsQuery {
    Crunchyroll,
    Funimation,
    Wakanim,
    Amazon,
    Hidive,
    Hulu,
    Youtube,
    Netflix,
}

#[derive(Deserialize, Clone, IntoStaticStr, Debug, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(Deserialize, Clone, IntoStaticStr, Debug, PartialEq)]
pub enum DelayedTimetable {
    Delayed,
    #[serde(rename = "On Break")]
    OnBreak,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct Timetables(pub Vec<TimetableAnime>);
impl Deref for Timetables {
    type Target = Vec<TimetableAnime>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Timetables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimetableAnime {
    /// The display title. Separate from other names and used as a high-priority name in some cases.
    pub title: String,
    /// The unique URL slug.
    pub route: String,
    /// The Romaji name.
    pub romaji: Option<String>,
    /// The English name.
    pub english: Option<String>,
    /// The Native name.
    pub native: Option<String>,
    /// The timetable delayed display text.
    pub delayed_text: Option<String>,
    /// The date from which it has been delayed.
    pub delayed_from: Option<String>,
    /// The date until it has been delayed to.
    pub delayed_until: Option<DateTime<Utc>>,
    /// The airing status.
    pub status: AirStatus,
    /// The episode's date and time.
    pub episode_date: DateTime<Utc>,
    /// The episode's number.
    pub episode_number: u64,
    /// The lowest episode number. Used only if multiple episodes air. The full format is SubtractedEpisodeNumber - EpisodeNumber.
    pub subtracted_episode_number: Option<u64>,
    /// The total episodes of an anime. 0 indicates unknown.
    pub episodes: Option<u64>,
    /// The length of an episode in minutes.
    pub length_min: u64,
    /// Whether a timetable anime is a donghua/chinese.
    pub donghua: bool,
    /// The air type. Raw and Dub will only display timetable anime that match it. Sub will use Raw if Sub is not available.
    pub air_type: AirType,
    /// The timetable anime's media types in an array of the category object.
    pub media_types: Vec<Category>,
    /// The timetable anime's poster/image URL slug.
    pub image_version_route: String,
    pub streams: Streams,
    /// The episode's immediate timetable status.
    pub airing_status: AiringStatus,
}

#[derive(Deserialize, Clone, IntoStaticStr, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AirType {
    Raw,
    Sub,
    Dub,
}

#[derive(Serialize, Copy, Clone, IntoStaticStr, Debug, PartialEq)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AirTypeQuery {
    Raw,
    Sub,
    Dub,
    All,
}

/// Immediate timetable status
#[derive(Deserialize, Clone, IntoStaticStr, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum AiringStatus {
    Airing,
    Aired,
    Unaired,
    DelayedAir,
}

/// Match type
#[derive(Serialize, Clone, IntoStaticStr, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MatchType {
    /// Any searches for anime that match any of the filters.
    Any,
    /// All searches for ones that match all of the filters.
    #[default]
    All,
}

/// Match type
#[derive(Serialize, Clone, IntoStaticStr, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortingType {
    #[default]
    Popularity,
    Score,
    Alphabetic,
    ReleaseDate,
}

/// Match type
#[derive(Serialize, Clone, IntoStaticStr, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SeasonQuery {
    Spring,
    Summer,
    Fall,
    Winter,
}

/// returns a datetime parsed into japan's fixedoffset
fn jpn_datetime_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(jpn_datetime(deserializer)?))
}

/// returns a datetime parsed into japan's fixedoffset
fn jpn_datetime<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    let japan_tz = FixedOffset::east_opt(9 * 3600).unwrap();

    let datetime = s
        .parse::<DateTime<Utc>>()
        .map_err(|e| Error::custom(e.to_string()))?;

    let datetime = datetime.with_timezone(&japan_tz);

    Ok(datetime)
}
