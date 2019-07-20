use actix_web::web;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::collections::BinaryHeap;
use tag_geotag::*;

mod load;

type HashMap<K, V> = load::HashMap<K, V>;

static TAGS: OnceCell<HashMap<String, Vec<u64>>> = OnceCell::new();
static GEOTAGS: OnceCell<HashMap<u64, GeoTag>> = OnceCell::new();
const ENTRY_COUNT: usize = 100;
const STRATEGY_BORDER: usize = 5000;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct QueryWrap {
    tag: String,
}

struct DataPair<'a> {
    id: u64,
    geotag: &'a GeoTag,
}

impl<'a> Ord for DataPair<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.geotag.time.cmp(&other.geotag.time)
    }
}

impl<'a> PartialOrd for DataPair<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.geotag.time.partial_cmp(&other.geotag.time)
    }
}

impl<'a> Eq for DataPair<'a> {}

impl<'a> PartialEq for DataPair<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.geotag.time == other.geotag.time
    }
}

fn query(q: web::Query<QueryWrap>) -> String {
    if let Some(i) = TAGS.get().unwrap().get(&q.tag) {
        if i.len() < STRATEGY_BORDER {
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
        } else {
            i.into_iter()
                .map(|id| DataPair {
                    id: *id,
                    geotag: &GEOTAGS.get().unwrap()[id],
                })
                .fold(
                    BinaryHeap::<std::cmp::Reverse<_>>::with_capacity(100),
                    |mut heap, e| {
                        if heap.len() == ENTRY_COUNT && e < heap.peek().unwrap().0 {
                            return heap;
                        }
                        heap.push(std::cmp::Reverse(e));
                        if heap.len() > ENTRY_COUNT {
                            heap.pop();
                        }
                        heap
                    },
                )
                .into_iter()
                .map(|t| t.0.geotag.to_csv_row(t.0.id))
                .collect::<Vec<_>>()
        }
        .join("")
    } else {
        "".into()
    }
}

fn main() {
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
