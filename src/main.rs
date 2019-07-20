use actix_web::web;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::io::BufRead;
use tag_geotag::*;

type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

const TAGS_SIZE: usize = 860621;
const GEOTAGS_SIZE: usize = 6145483;

static TAGS: OnceCell<HashMap<String, Vec<u64>>> = OnceCell::new();
static GEOTAGS: OnceCell<HashMap<u64, GeoTag>> = OnceCell::new();
static BASE_DIR: OnceCell<String> = OnceCell::new();

fn load_tags(filename: &str) -> HashMap<String, Vec<u64>> {
    let f = std::fs::File::open(&format!("{}/{}", BASE_DIR.get().unwrap(), filename)).unwrap();
    let r = std::io::BufReader::new(f);

    let mut tags = HashMap::with_capacity_and_hasher(TAGS_SIZE, Default::default());
    // Note that tag_pp.csv has "NO_TAG" at the first line
    for s in r.lines().skip(1) {
        let mut s = s.unwrap();
        if s.ends_with('\n') {
            s.pop();
        }
        let mut sp = s.split(',');
        let key = sp.next().unwrap();
        // skip the size column
        tags.insert(
            key.to_owned(),
            sp.skip(1).map(|s| s.parse().unwrap()).collect::<Vec<_>>(),
        );
    }

    for v in tags.values_mut() {
        v.sort();
    }
    tags
}

fn load_geotags(filename: &str) -> HashMap<u64, GeoTag> {
    let mut geotags = HashMap::with_capacity_and_hasher(GEOTAGS_SIZE, Default::default());

    let f = std::fs::File::open(&format!("{}/{}", BASE_DIR.get().unwrap(), filename)).unwrap();
    let r = std::io::BufReader::new(f);

    for s in r.lines() {
        let mut s = s.unwrap();
        if s.ends_with('\n') {
            s.pop();
        }
        let ret = GeoTag::from_str_to_geotag(&s).expect(&s);
        geotags.insert(ret.0, ret.1);
    }
    geotags
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct QueryWrap {
    tag: String,
}

fn query(q: web::Query<QueryWrap>) -> String {
    if let Some(i) = TAGS.get().unwrap().get(&q.tag) {
        let mut v = i
            .iter()
            .map(|id| (id, &GEOTAGS.get().unwrap()[id]))
            .collect::<Vec<_>>();
        v.sort_unstable_by(|a, b| a.1.time.cmp(&b.1.time).reverse());
        v.into_iter()
            .take(100)
            .map(|t| t.1.to_csv_row(*t.0))
            .collect::<Vec<_>>()
            .join("")
    } else {
        "".into()
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let _ = BASE_DIR.set(args.next().unwrap());
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
    let h1 = std::thread::spawn(|| TAGS.set(load_tags("tag_pp.csv")));
    let h2 = std::thread::spawn(|| GEOTAGS.set(load_geotags("geotag_pp.csv")));
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
