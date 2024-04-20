use const_format::formatcp;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    errors::ApiError,
    objects::{
        AirStatusQuery, Anime, AnimePage, MatchType, SeasonQuery, SortingType, StreamsQuery,
    },
    rate_limit::RateLimit,
    utils::IsJson as _,
    Client, API_URL,
};

const API_ANIME: &str = formatcp!("{API_URL}/anime");
const API_ANIME_SLUG: &str = formatcp!("{API_URL}/anime/{{slug}}");

pub struct AnimeApi {
    client: Client,
}

impl AnimeApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn get(&self) -> AnimeGet {
        AnimeGet {
            client: self.client.clone(),
            page: None,
            q: None,
            mt: None,
            st: None,
            genres: None,
            genres_exclude: None,
            studios: None,
            studios_exclude: None,
            sources: None,
            sources_exclude: None,
            media_types: None,
            media_types_exclude: None,
            years: None,
            years_exclude: None,
            seasons: None,
            seasons_exclude: None,
            airing_statuses: None,
            airing_statuses_exclude: None,
            duration: None,
            episodes: None,
            streams: None,
            streams_exclude: None,
            mal_ids: None,
            anilist_ids: None,
            anidb_ids: None,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AnimeGet {
    #[serde(skip)]
    client: Client,

    /// The number of the page of the anime array being requested. Defaults to 1.
    page: Option<u64>,
    /// Filter by text. Applies to an anime's names. Failing that it tries genres, studios,
    /// sources and media types. Maximum length is 200.
    q: Option<String>,
    /// The filter match type you want to use. Valid values are any and all. Any searches for anime
    /// that match any of the filters. All searches for ones that match all of the filters. Defaults to all.
    mt: Option<MatchType>,
    /// The sorting type you want to use. Valid values are popularity, score, alphabetic and releaseDate.
    /// Defaults to popularity.
    st: Option<SortingType>,
    /// Filter by genre. Requires the genre's route/slug. For multiple genres add another genres query for
    /// each genre.
    genres: Option<Vec<String>>,
    /// Exclude a genre from the search. Requires the genre's route/slug. For multiple genres add another
    /// genres-exclude
    /// query for each genre.
    genres_exclude: Option<Vec<String>>,
    /// Filter by genre. Requires the studio's route/slug. For multiple studios add another studios query
    /// for each studio.
    studios: Option<Vec<String>>,
    /// Exclude a studio from the search. Requires the studio's route/slug. For multiple studios add another
    /// studios-exclude
    /// query for each studio.
    studios_exclude: Option<Vec<String>>,
    /// Filter by genre. Requires the source's route/slug. For multiple sources add another sources query
    /// for each source.
    sources: Option<Vec<String>>,
    /// Exclude a source from the search. Requires the source's route/slug. For multiple sources add another
    /// sources-exclude query for each source.
    sources_exclude: Option<Vec<String>>,
    /// Filter by genre. Requires the media type's route/slug. For multiple media types add another media-types
    /// query for each media-type.
    media_types: Option<Vec<String>>,
    /// Exclude a media type from the search. Requires the media type's route/slug. For multiple media types
    /// add another media-types-exclude query for each media type.
    media_types_exclude: Option<Vec<String>>,
    /// Filter by year. For multiple years add another years query for each year.
    years: Option<Vec<u16>>,
    /// Exclude a year from the search. For multiple years add another years-exclude query for each year.
    years_exclude: Option<Vec<u16>>,
    /// Filter by calendar season. For multiple seasons add another seasons query for each season.
    seasons: Option<Vec<SeasonQuery>>,
    /// Exclude a calendar season from the search. For multiple seasons add another seasons-exclude query
    /// for each season.
    seasons_exclude: Option<Vec<SeasonQuery>>,
    /// Filter by airing status. For multiple airing statuses add another airing-statuses query for each
    /// airing status. Valid values are finished, ongoing and upcoming.
    airing_statuses: Option<Vec<AirStatusQuery>>,
    /// Exclude an airing status from the search. For multiple airing statuses add another airing-statuses-exclude
    /// query for each airing status. Valid values are finished, ongoing and upcoming.
    airing_statuses_exclude: Option<Vec<AirStatusQuery>>,
    /// Filter by episode duration (in minutes.) Format is 0-∞.
    duration: Option<String>,
    /// Filter by episode count. Format is 0-∞.
    episodes: Option<String>,
    /// Filter by stream. Requires the stream's name. For multiple streams add another streams query for each stream.
    streams: Option<Vec<StreamsQuery>>,
    /// Exclude a stream from the search. Requires the stream's name. For multiple streams add another
    /// streams-exclude query for each stream.
    streams_exclude: Option<Vec<StreamsQuery>>,
    /// Filter by MyAnimeList ID. For multiple ids add another mal-ids query for each id.
    mal_ids: Option<Vec<u64>>,
    /// Filter by AniList ID. For multiple ids add another anilist-ids query for each id.
    anilist_ids: Option<Vec<u64>>,
    /// Filter by AniDB ID. For multiple ids add another anidb-ids query for each id.
    anidb_ids: Option<Vec<u64>>,
}

impl AnimeGet {
    /// Fetch the data of a specific anime
    pub fn slug(&self, slug: &str) -> AnimeSlug {
        AnimeSlug {
            client: self.client.clone(),
            slug: slug.to_owned(),
        }
    }

    /// Fetches a paginated array of anime. Each page can contain up to 18 anime. Can be filtered further by using
    /// the optional parameters. Contains the page number and
    /// amount of anime that match the parameters.
    pub fn page(mut self, page: u64) -> Self {
        self.page = Some(page.clamp(1, u64::MAX));
        self
    }

    /// Filter by text. Applies to an anime's names. Failing that it tries genres, studios, sources and media types.
    /// Maximum length is 200.
    pub fn q(mut self, q: &str) -> Self {
        let mut q = q.to_owned();
        q.truncate(200);

        self.q = Some(q);
        self
    }

    /// The filter match type you want to use. Valid values are any and all. Any searches for anime that match any of
    /// the filters. All searches for ones that match all of the filters. Defaults to all.
    pub fn mt(mut self, mt: MatchType) -> Self {
        self.mt = Some(mt);
        self
    }

    /// The sorting type you want to use. Valid values are popularity, score, alphabetic and releaseDate. Defaults
    /// to popularity.
    pub fn st(mut self, st: SortingType) -> Self {
        self.st = Some(st);
        self
    }

    /// Filter by genre. Requires the genre's route/slug. For multiple genres add another genres query for each genre.
    pub fn genres<'a, I: IntoIterator<Item = &'a str>>(mut self, genres: I) -> Self {
        self.genres = Some(genres.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Exclude a genre from the search. Requires the genre's route/slug. For multiple genres add another genres-exclude
    /// query for each genre.
    pub fn genres_exclude<'a, I: IntoIterator<Item = &'a str>>(mut self, genres: I) -> Self {
        self.genres_exclude = Some(genres.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Filter by genre. Requires the studio's route/slug. For multiple studios add another studios query for each studio.
    pub fn studios<'a, I: IntoIterator<Item = &'a str>>(mut self, studios: I) -> Self {
        self.studios = Some(studios.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Exclude a studio from the search. Requires the studio's route/slug. For multiple studios add another studios-exclude
    /// query for each studio.
    pub fn studios_exclude<'a, I: IntoIterator<Item = &'a str>>(mut self, studios: I) -> Self {
        self.studios_exclude = Some(studios.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Filter by genre. Requires the source's route/slug. For multiple sources add another sources query for each source.
    pub fn sources<'a, I: IntoIterator<Item = &'a str>>(mut self, sources: I) -> Self {
        self.sources = Some(sources.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Exclude a source from the search. Requires the source's route/slug. For multiple sources add another sources-exclude
    /// query for each source.
    pub fn sources_exclude<'a, I: IntoIterator<Item = &'a str>>(mut self, sources: I) -> Self {
        self.sources_exclude = Some(sources.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Filter by genre. Requires the media type's route/slug. For multiple media types add another media-types query for each
    /// media-type.
    pub fn media_types<'a, I: IntoIterator<Item = &'a str>>(mut self, media_types: I) -> Self {
        self.media_types = Some(media_types.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Exclude a media type from the search. Requires the media type's route/slug. For multiple media types add another
    /// media-types-exclude query for each media type.
    pub fn media_types_exclude<'a, I: IntoIterator<Item = &'a str>>(
        mut self,
        media_types: I,
    ) -> Self {
        self.media_types_exclude = Some(media_types.into_iter().map(|s| s.to_owned()).collect());
        self
    }

    /// Filter by year. For multiple years add another years query for each year.
    pub fn years<I: IntoIterator<Item = u16>>(mut self, years: I) -> Self {
        self.years = Some(years.into_iter().collect());
        self
    }

    /// Exclude a year from the search. For multiple years add another years-exclude query for each year.
    pub fn years_exclude<I: IntoIterator<Item = u16>>(mut self, years: I) -> Self {
        self.years_exclude = Some(years.into_iter().collect());
        self
    }

    /// Filter by calendar season. For multiple seasons add another seasons query for each season.
    pub fn seasons<I: IntoIterator<Item = SeasonQuery>>(mut self, seasons: I) -> Self {
        self.seasons = Some(seasons.into_iter().collect());
        self
    }

    /// Exclude a calendar season from the search. For multiple seasons add another seasons-exclude query for each season.
    pub fn seasons_exclude<I: IntoIterator<Item = SeasonQuery>>(mut self, seasons: I) -> Self {
        self.seasons_exclude = Some(seasons.into_iter().collect());
        self
    }

    /// Filter by airing status. For multiple airing statuses add another airing-statuses query for each airing status.
    /// Valid values are finished, ongoing and upcoming.
    pub fn airing_statuses<I: IntoIterator<Item = AirStatusQuery>>(mut self, seasons: I) -> Self {
        self.airing_statuses = Some(seasons.into_iter().collect());
        self
    }

    /// Exclude an airing status from the search. For multiple airing statuses add another airing-statuses-exclude query
    /// for each airing status. Valid values are finished, ongoing and upcoming.
    pub fn airing_statuses_exclude<I: IntoIterator<Item = AirStatusQuery>>(
        mut self,
        seasons: I,
    ) -> Self {
        self.airing_statuses_exclude = Some(seasons.into_iter().collect());
        self
    }

    /// Filter by episode duration (in minutes.) Format is 0-∞.
    pub fn duration(mut self, duration: &str) -> Self {
        self.duration = Some(duration.to_owned());
        self
    }

    /// Filter by episode count. Format is 0-∞.
    pub fn episodes(mut self, episodes: &str) -> Self {
        self.episodes = Some(episodes.to_owned());
        self
    }

    /// Filter by stream. Requires the stream's name. For multiple streams add another streams query for each stream.
    pub fn streams<I: IntoIterator<Item = StreamsQuery>>(mut self, streams: I) -> Self {
        self.streams = Some(streams.into_iter().collect());
        self
    }

    /// Exclude a stream from the search. Requires the stream's name. For multiple streams add another streams-exclude
    /// query for each stream.
    pub fn streams_exclude<I: IntoIterator<Item = StreamsQuery>>(mut self, streams: I) -> Self {
        self.streams_exclude = Some(streams.into_iter().collect());
        self
    }

    /// Filter by MyAnimeList ID. For multiple ids add another mal-ids query for each id.
    pub fn mal_ids<I: IntoIterator<Item = u64>>(mut self, mal_ids: I) -> Self {
        self.mal_ids = Some(mal_ids.into_iter().collect());
        self
    }

    /// Filter by AniList ID. For multiple ids add another anilist-ids query for each id.
    pub fn anilist_ids<I: IntoIterator<Item = u64>>(mut self, anilist_ids: I) -> Self {
        self.anilist_ids = Some(anilist_ids.into_iter().collect());
        self
    }

    /// Filter by AniDB ID. For multiple ids add another anidb-ids query for each id.
    pub fn anidb_ids<I: IntoIterator<Item = u64>>(mut self, anidb_ids: I) -> Self {
        self.anidb_ids = Some(anidb_ids.into_iter().collect());
        self
    }

    pub async fn send(self) -> Result<(RateLimit, AnimePage), ApiError> {
        let query = serde_qs::to_string(&self).unwrap();

        let url = format!("{API_ANIME}?{query}");

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.token.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let text = response.text().await?;

        if !text.is_json() {
            return Err(ApiError::Api(text));
        }

        let page: AnimePage = serde_json::from_str(&text)?;

        Ok((limit, page))
    }
}

/// Fetch the data of a specific anime
pub struct AnimeSlug {
    client: Client,
    slug: String,
}

impl AnimeSlug {
    pub async fn send(self) -> Result<(RateLimit, Anime), ApiError> {
        let url = API_ANIME_SLUG.replace("{slug}", &self.slug);

        let response = self
            .client
            .http
            .get(url)
            .bearer_auth(self.client.token.app_token())
            .send()
            .await?;

        let headers = response.headers();
        let limit = RateLimit::new(headers);

        let text = response.text().await?;

        if !text.is_json() {
            return Err(ApiError::Api(text));
        }

        let anime: Anime = serde_json::from_str(&text)?;

        Ok((limit, anime))
    }
}
