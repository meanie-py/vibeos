//! A bounded cache for legacy rules that construct patterns from symbol names.
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};

#[derive(Default)]
struct Cache {
    patterns: HashMap<String, Regex>,
    order: VecDeque<String>,
}
static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| Mutex::new(Cache::default()));

pub fn cached(pattern: &str) -> Result<Regex, regex::Error> {
    if let Some(found) = CACHE
        .lock()
        .expect("regex cache poisoned")
        .patterns
        .get(pattern)
    {
        return Ok(found.clone());
    }
    // Compilation occurs outside the lock so unrelated rules can make progress.
    let compiled = Regex::new(pattern)?;
    let mut cache = CACHE.lock().expect("regex cache poisoned");
    if let Some(found) = cache.patterns.get(pattern) {
        return Ok(found.clone());
    }
    if cache.patterns.len() >= 8192 {
        if let Some(old) = cache.order.pop_front() {
            cache.patterns.remove(&old);
        }
    }
    cache.order.push_back(pattern.into());
    cache.patterns.insert(pattern.into(), compiled.clone());
    Ok(compiled)
}
