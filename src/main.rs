mod load;
mod structs;

use actix_web::{web, HttpResponse};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use structs::*;
use tag_geotag::*;

#[cfg(feature = "cache")]
mod cache;

#[cfg(feature = "cache")]
const CACHE_LENGTH: usize = 100;

#[cfg(feature = "cache")]
static CACHE: once_cell::sync::Lazy<std::sync::RwLock<cache::Cache>> =
    once_cell::sync::Lazy::new(|| std::sync::RwLock::new(cache::Cache::new(CACHE_LENGTH)));

type HashMap<K, V> = load::HashMap<K, V>;

static TAGS: OnceCell<HashMap<String, Vec<u64>>> = OnceCell::new();
static GEOTAGS: OnceCell<HashMap<u64, GeoTag>> = OnceCell::new();
const ENTRY_COUNT: usize = 100;

// This SHOULD be equal or more than ENTRY_COUNT
const STRATEGY_BORDER: usize = 5000;

#[derive(Deserialize, Clone, Debug)]
struct QueryWrap {
    tag: String,
    strategy: Option<SortStrategy>,
    #[cfg(feature = "cache")]
    cache: Option<bool>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug, Clone, Copy)]
enum SortStrategy {
    VecSort,
    Heap,
    HeapNeu,
}

fn top_n_vec_sort<'a, I: Iterator<Item = DataPair<'a>>>(dp: I) -> Vec<DataPair<'a>> {
    // fetch all data, sort them, and take the needed elements
    let mut v = dp.collect::<Vec<_>>();
    v.sort_unstable_by(|a, b| a.cmp(&b).reverse());
    v.into_iter().take(ENTRY_COUNT).collect()
}

fn top_n_heap<'a, I: Iterator<Item = DataPair<'a>>>(dp: I) -> Vec<DataPair<'a>> {
    // fetch data, put it into the heap, then take all and sort them
    dp.fold(
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
    .map(|e| e.0)
    .collect()
}

fn top_n_heap_neu<'a, I: Iterator<Item = DataPair<'a>>>(dp: I) -> Vec<DataPair<'a>> {
    let mut dp = dp;
    let mut heap: BinaryHeap<_> = (&mut dp).take(ENTRY_COUNT).map(Reverse).collect();
    let mut guard = &heap.peek().unwrap().0;
    for e in dp {
        if e > *guard {
            {
                *heap.peek_mut().unwrap() = Reverse(e);
            }
            guard = &heap.peek().unwrap().0;
        }
    }
    heap.into_sorted_vec().into_iter().map(|e| e.0).collect()
}

fn generate_html(data: Vec<DataPair>) -> String {
    data.into_iter().fold(r#"<!doctype html><html><head><title>超高性能化</title><meta charset="utf-8"></head><body>"#.to_string(), |s, x| {
        s + &format!("<div><img src={} alt={}><p>Latitude : {}<br>Longitude : {}<br>Shot at {}</p></div>", x.geotag.get_url(x.id), x.id, x.geotag.latitude, x.geotag.longitude, chrono::NaiveDateTime::from_timestamp(x.geotag.time as i64, 0))
    }) + "</body></html>"
}

fn query(q: web::Query<QueryWrap>) -> HttpResponse {
    let tag = &q.tag;
    let mut response = HttpResponse::Ok();
    response.content_type("text/html");

    #[cfg(feature = "cache")]
    let use_cache = q.cache.unwrap_or(true);
    #[cfg(feature = "cache")]
    {
        if use_cache {
            if let Some(i) = CACHE.read().unwrap().get_cache(&tag) {
                return response.body(i);
            }
        }
    }

    if let Some(i) = TAGS.get().unwrap().get(tag) {
        let strat = q.strategy.unwrap_or_else(|| {
            if i.len() < STRATEGY_BORDER {
                SortStrategy::VecSort
            } else {
                SortStrategy::Heap
            }
        });

        let it = i.iter().map(|id| DataPair {
            id: *id,
            geotag: &GEOTAGS.get().unwrap()[&id],
        });

        let s = generate_html(match strat {
            SortStrategy::VecSort => top_n_vec_sort(it),
            SortStrategy::Heap => top_n_heap(it),
            SortStrategy::HeapNeu => top_n_heap_neu(it),
        });

        #[cfg(feature = "cache")]
        {
            if use_cache {
                CACHE.write().unwrap().push_cache(tag, &s);
            }
        }

        return response.body(s);
    }

    response.body(r#"<!doctype html><html><head><title>超高性能化</title><meta charset="utf-8"></head><body><p>No Match</p></body></html>"#)
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
