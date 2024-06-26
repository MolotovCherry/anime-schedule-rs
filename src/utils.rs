pub trait IsJson {
    fn is_json(&self) -> bool;
}

impl IsJson for String {
    fn is_json(&self) -> bool {
        serde_json::from_str::<serde::de::IgnoredAny>(self.as_str()).is_ok()
    }
}

pub struct LazyLock<T, F = fn() -> T> {
    data: ::std::sync::OnceLock<T>,
    f: F,
}

impl<T, F> LazyLock<T, F> {
    pub const fn new(f: F) -> LazyLock<T, F> {
        Self {
            data: ::std::sync::OnceLock::new(),
            f,
        }
    }
}

impl<T> ::std::ops::Deref for LazyLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.get_or_init(self.f)
    }
}
