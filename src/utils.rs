pub trait IsJson {
    fn is_json(&self) -> bool;
}

impl IsJson for String {
    fn is_json(&self) -> bool {
        serde_json::from_str::<serde::de::IgnoredAny>(self.as_str()).is_ok()
    }
}
