use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::RwLock;

pub const CACHE_LENGTH: usize = 100;

pub static CACHE: Lazy<RwLock<CacheContainer>> = Lazy::new(|| RwLock::new(CacheContainer::new(CACHE_LENGTH)));

pub struct Cache {
    pub tag: String,
    pub content: String,
}

pub struct CacheContainer {
    data: VecDeque<Cache>,
    capacity: usize,
}

pub fn init() {
    Lazy::force(&CACHE);
}

impl CacheContainer {
    pub fn new(capacity: usize) -> Self {
        CacheContainer {
            data: VecDeque::with_capacity(capacity),
            capacity: capacity,
        }
    }

    pub fn get(&self, tag: &str) -> Option<&Cache> {
        self.data
            .iter()
            .find(|e| e.tag == tag)
    }

    pub fn push(&mut self, c: Cache) {
        println!("New tag in cache: {}", c.tag);
        if self.data.len() == self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(c);
    }
}
