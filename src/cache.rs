use crate::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

pub const CACHE_LENGTH: usize = 100;

pub static CACHE: Lazy<RwLock<CacheContainer>> =
    Lazy::new(|| RwLock::new(CacheContainer::new(CACHE_LENGTH)));

/*
head(old) |--[elem1]--[elem2]--| tail(new)
 */
/// HashMap with FIFO double-linked list
pub struct CacheContainer {
    data: HashMap<String, *const Cache>,
    capacity: usize,
    head: *mut Cache,
    tail: *mut Cache,
}

// XXX: really?
unsafe impl Sync for CacheContainer {}
unsafe impl Send for CacheContainer {}

struct Cache {
    tag: String,
    content: String,
    prev: *mut Cache, // older
    next: *mut Cache, // newer
}

pub fn init() {
    Lazy::force(&CACHE);
}

impl CacheContainer {
    /// Construct new empty cache. Panics if capacity is 0.
    pub fn new(capacity: usize) -> Self {
        assert_ne!(capacity, 0);
        CacheContainer {
            data: HashMap::with_capacity_and_hasher(capacity, Default::default()),
            capacity: capacity,
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }

    pub fn get(&self, tag: &str) -> Option<String> {
        self.data.get(tag).map(|c| unsafe { &**c }.content.clone())
    }

    /// Add a new element to the cache, and also removes the oldest element
    /// when the cache is full.
    pub fn push(&mut self, tag: String, content: String) {
        println!(
            "New tag in cache of {:?}: {}",
            std::thread::current().id(),
            tag
        );
        if self.data.len() == self.capacity {
            self.pop_front();
        }
        self.push_back(tag, content);
    }

    /// Remove the oldest node.
    fn pop_front(&mut self) {
        // SAFETY: self.head is not a nullptr, because this function is never
        // called by anyone other than push and thus there is at least
        // one element when this is called.
        let c = unsafe { &*self.head };

        self.data.remove(&c.tag);

        // clean list up
        // SAFETY: I think this is safe :(
        unsafe {
            let next = c.next;
            (*next).prev = std::ptr::null_mut();
            self.head.drop_in_place();
            self.head = next;
        }
    }

    fn push_back(&mut self, tag: String, content: String) {
        let node = Box::into_raw(Box::new(Cache {
            tag: tag.clone(),
            content: content,
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        }));

        // add node to list
        // SAFETY: I think this is safe :(
        unsafe {
            if self.head == std::ptr::null_mut() {
                self.head = node;
            } else {
                (*self.tail).next = node;
            }
            self.tail = node;
        }

        self.data.insert(tag, node);
    }
}
