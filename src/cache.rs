use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::RwLock;

const CACHE_LENGTH: usize = 100;

static CACHE: Lazy<RwLock<Cache>> = Lazy::new(|| RwLock::new(Cache::new(CACHE_LENGTH)));

pub struct CacheWrap {
    pub tag: String,
    pub content: String,
}

pub struct Cache {
    data: VecDeque<CacheWrap>,
    capacity: usize,
}

pub fn init() {
    Lazy::force(&CACHE);
}

pub fn get_cache(tag: &str) -> Option<String> {
    CACHE.read().unwrap().get_cache(&tag)
}

pub fn push_cache(tag: &str, entry: &str) {
    CACHE.write().unwrap().push_cache(tag, entry)
}

impl Cache {
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: VecDeque::with_capacity(capacity),
            capacity: capacity,
        }
    }

    pub fn get_cache(&self, tag: &str) -> Option<String> {
        self.data
            .iter()
            .find(|e| e.tag == tag)
            .map(|cw| cw.content.clone())
    }

    pub fn push_cache(&mut self, tag: &str, entry: &str) {
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(CacheWrap {
            tag: tag.to_owned(),
            content: entry.to_owned(),
        });
    }
}
