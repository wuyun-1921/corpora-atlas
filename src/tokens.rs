use std::borrow::Cow;
use std::collections::HashMap;

use crate::config;

pub fn resolve_zim(shorthand: &str) -> Cow<'_, str> {
    config::Config::global()
        .kiwix
        .get(shorthand)
        .map_or(Cow::Borrowed(shorthand), |v| Cow::Owned(v.clone()))
}

pub fn resolve_slob(shorthand: &str) -> Cow<'_, str> {
    config::Config::global()
        .aard2
        .get(shorthand)
        .map_or(Cow::Borrowed(shorthand), |v| Cow::Owned(v.clone()))
}

pub fn get_fallbacks() -> &'static HashMap<String, Vec<String>> {
    &config::Config::global().fallback
}
