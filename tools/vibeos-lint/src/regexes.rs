//! A bounded cache for legacy rules that construct patterns from symbol names.
use regex::Regex;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};

#[derive(Default)]
struct Cache {
    patterns: HashMap<String, Regex>,
    order: VecDeque<String>,
}
static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| Mutex::new(Cache::default()));

thread_local! {
    // Rules repeatedly ask for a handful of fixed patterns while scanning lines.
    // Keep these clones local so hot hits never contend on the shared cache lock.
    static LOCAL: RefCell<Cache> = RefCell::new(Cache::default());
}

pub fn cached(pattern: &str) -> Result<Regex, regex::Error> {
    if let Some(found) = LOCAL.with(|cache| cache.borrow().patterns.get(pattern).cloned()) {
        return Ok(found);
    }
    let compiled = shared(pattern)?;
    LOCAL.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.patterns.len() >= 64 {
            if let Some(old) = cache.order.pop_front() {
                cache.patterns.remove(&old);
            }
        }
        cache.order.push_back(pattern.into());
        cache.patterns.insert(pattern.into(), compiled.clone());
    });
    Ok(compiled)
}

fn shared(pattern: &str) -> Result<Regex, regex::Error> {
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
