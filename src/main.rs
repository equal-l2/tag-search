use actix_web::web;
use failure::Fallible;
use once_cell::sync::OnceCell;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::io::BufRead;
use tag_geotag::*;

const TAGS_SIZE: usize = 860621;
const GEOTAGS_SIZE: usize = 6145483;

static TAGS: OnceCell<FxHashMap<String, Vec<u64>>> = OnceCell::new();
static GEOTAGS: OnceCell<FxHashMap<u64, GeoTag>> = OnceCell::new();
static BASE_DIR: OnceCell<String> = OnceCell::new();

fn from_str_to_geotag(s: &str) -> Fallible<(u64, GeoTag)> {
    let mut s = s.split(',');
    let id = s.next().ok_or(failure::err_msg("Id missing"))?.parse()?;
    let time = s.next().ok_or(failure::err_msg("Time missing"))?.parse()?;
    let latitude = s
        .next()
        .ok_or(failure::err_msg("Latitude missing"))?
        .parse()?;
    let longitude = s
        .next()
        .ok_or(failure::err_msg("Longitude missing"))?
        .parse()?;
    let domain_num = s
        .next()
        .ok_or(failure::err_msg("Serv_num missing"))?
        .chars()
        .nth(0)
        .ok_or(failure::err_msg("Invalid String"))?;
    let url_num1 = s
        .next()
        .ok_or(failure::err_msg("Url_num1 missing"))?
        .parse()?;
    let url_num2 = u64::from_str_radix(s.next().ok_or(failure::err_msg("Url_num2 missing"))?, 16)?;

    Ok((
        id,
        GeoTag {
            time,
            latitude,
            longitude,
            domain_num,
            url_num1,
            url_num2,
        },
    ))
}

fn load_tags(filename: &str) -> FxHashMap<String, Vec<u64>> {
    let f = std::fs::File::open(&format!("{}/{}", BASE_DIR.get().unwrap(), filename)).unwrap();
    let r = std::io::BufReader::new(f);

    let mut tags = FxHashMap::with_capacity_and_hasher(TAGS_SIZE, Default::default());
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

fn load_geotags(filename: &str) -> FxHashMap<u64, GeoTag> {
    let mut geotags = FxHashMap::with_capacity_and_hasher(GEOTAGS_SIZE, Default::default());

    let f = std::fs::File::open(&format!("{}/{}", BASE_DIR.get().unwrap(), filename)).unwrap();
    let r = std::io::BufReader::new(f);

    for s in r.lines() {
        let mut s = s.unwrap();
        if s.ends_with('\n') {
            s.pop();
        }
        let ret = from_str_to_geotag(&s).expect(&s);
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
    let _ = BASE_DIR.set(std::env::args().nth(1).unwrap());
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
    .bind(bind_addr)
    .unwrap()
    .run();
}
