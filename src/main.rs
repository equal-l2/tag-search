mod load;
mod structs;

use actix_web::web;
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::RwLock;
use serde::Deserialize;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

use structs::*;

type HashMap<K, V> = load::HashMap<K, V>;

static TAGS: OnceCell<HashMap<String, Vec<u64>>> = OnceCell::new();
static GEOTAGS: OnceCell<HashMap<u64, tag_geotag::GeoTag>> = OnceCell::new();
static CACHE: Lazy<RwLock<VecDeque<CacheWrap>>> =
    Lazy::new(|| RwLock::new(VecDeque::with_capacity(CACHE_LENGTH)));
const ENTRY_COUNT: usize = 100;

// This SHOULD be equal or more than ENTRY_COUNT
const STRATEGY_BORDER: usize = 5000;

const CACHE_LENGTH: usize = 100;

fn get_cache(tag: &str) -> Option<String> {
    CACHE
        .read()
        .iter()
        .find(|&e| &e.tag == tag)
        .map(|cw| cw.content.clone())
}

#[derive(Deserialize, Clone, Debug)]
struct QueryWrap {
    tag: String,
    cache: Option<bool>,
    strat: Option<SortStrategy>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug, Clone, Copy)]
enum SortStrategy {
    VecSort,
    HeapPushPop,
    HeapPeek,
}

fn query(q: web::Query<QueryWrap>) -> String {
    let tag = &q.tag;
    let use_cache = q.cache.unwrap_or(true);

    if use_cache {
        if let Some(i) = get_cache(tag) {
            return i;
        }
    }

    if let Some(i) = TAGS.get().unwrap().get(tag) {
        let strat = q.strat.unwrap_or_else(|| {
            if i.len() < STRATEGY_BORDER {
                SortStrategy::VecSort
            } else {
                SortStrategy::HeapPushPop
            }
        });

        let s = match strat {
            SortStrategy::VecSort => {
                // fetch all data, sort them, and take the needed elements
                let mut v = i
                    .into_iter()
                    .map(|id| DataPair {
                        id: *id,
                        geotag: &GEOTAGS.get().unwrap()[id],
                    })
                    .collect::<Vec<_>>();
                v.sort_unstable_by(|a, b| a.cmp(&b).reverse());
                v.into_iter()
                    .take(ENTRY_COUNT)
                    .map(|t| t.geotag.to_csv_row(t.id))
                    .collect::<Vec<_>>()
            }
            SortStrategy::HeapPushPop => {
                // fetch data, put it into the heap, then take all and sort them
                i.into_iter()
                    .map(|id| DataPair {
                        id: *id,
                        geotag: &GEOTAGS.get().unwrap()[id],
                    })
                    .fold(
                        BinaryHeap::<Reverse<_>>::with_capacity(ENTRY_COUNT),
                        |mut heap, e| {
                            if heap.len() == ENTRY_COUNT && e <= heap.peek().unwrap().0 {
                                return heap;
                            }
                            heap.push(Reverse(e));
                            if heap.len() > ENTRY_COUNT {
                                heap.pop();
                            }
                            heap
                        },
                    )
                    .into_sorted_vec()
                    .into_iter()
                    .map(|t| t.0.geotag.to_csv_row(t.0.id))
                    .collect::<Vec<_>>()
            }
            SortStrategy::HeapPeek => {
                // fetch data, put it into the heap, then take all and sort them
                i.into_iter()
                    .map(|id| DataPair {
                        id: *id,
                        geotag: &GEOTAGS.get().unwrap()[id],
                    })
                    .fold(
                        BinaryHeap::<Reverse<_>>::with_capacity(ENTRY_COUNT),
                        |mut heap, e| {
                            if heap.len() == ENTRY_COUNT {
                                let mut top = heap.peek_mut().unwrap();
                                if e > top.0 {
                                    *top = Reverse(e);
                                }
                            } else {
                                heap.push(Reverse(e));
                            }
                            heap
                        },
                    )
                    .into_sorted_vec()
                    .into_iter()
                    .map(|t| t.0.geotag.to_csv_row(t.0.id))
                    .collect::<Vec<_>>()
            }
        }
        .join("");
        if use_cache {
            let mut cache = CACHE.write();
            if cache.len() >= CACHE_LENGTH {
                cache.pop_front();
            }
            cache.push_back(CacheWrap {
                tag: tag.clone(),
                content: s.clone(),
            });
        }
        return s;
    }

    "".into()
}

fn main() {
    // parse args
    let mut args = std::env::args().skip(1);
    let _ = load::BASE_DIR.set(args.next().unwrap());
    let worker_num = {
        let n = args.next();
        if n.is_none() {
            let n_cpu = num_cpus::get();
            println!("Worker count unspecified, use {} worker(s)", n_cpu);
            n_cpu
        } else {
            let n = n.unwrap().parse().expect("2nd arg must be a number");
            if n == 0 {
                let n_cpu = num_cpus::get();
                println!("workers unspecified, use {} worker(s)", n_cpu);
                n_cpu
            } else {
                println!("Use {} worker(s)", n);
                n
            }
        }
    };

    // initialize global variables
    println!("Now loading... (Wait patiently)");
    let now = std::time::Instant::now();
    let h1 = std::thread::spawn(|| TAGS.set(load::load_tags("tag_pp.csv")));
    let h2 = std::thread::spawn(|| GEOTAGS.set(load::load_geotags("geotag_pp.csv")));
    h1.join().unwrap().unwrap();
    println!("Tags size : {}", TAGS.get().unwrap().len());
    h2.join().unwrap().unwrap();
    println!("Geotags size : {}", GEOTAGS.get().unwrap().len());
    println!(
        "Load complete, elapsed time : {}[s]",
        (now.elapsed().as_millis() as f64) / 1000f64
    );

    // start the web server
    let bind_addr = "0.0.0.0:8080";
    println!("Server listening at '{}'", bind_addr);
    let _ = actix_web::HttpServer::new(|| {
        actix_web::App::new().service(web::resource("query.html").to(query))
    })
    .workers(worker_num)
    .bind(bind_addr)
    .unwrap()
    .run();
}
