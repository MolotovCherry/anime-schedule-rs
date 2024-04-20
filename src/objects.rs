mod account;
mod anime;
mod lists;

use std::ops::{Deref, DerefMut};

pub use account::*;
pub use anime::*;
pub use lists::*;

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
