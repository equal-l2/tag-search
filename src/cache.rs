use std::collections::VecDeque;

pub struct CacheWrap {
    pub tag: String,
    pub content: String,
}

pub struct Cache {
    data: VecDeque<CacheWrap>,
    capacity: usize,
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
