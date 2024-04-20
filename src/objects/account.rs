use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct UserCategoryStat {
    pub route: String,
    pub name: String,
    pub amount: u64,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub user_id: String,
    pub days_anime_seen: f64,
    pub average_anime_score: f64,
    pub user_genre_stats: HashMap<String, UserCategoryStat>,
    pub user_studio_stats: HashMap<String, UserCategoryStat>,
}
