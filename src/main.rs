use actix_web::web;
use chrono::NaiveDateTime;
use failure::Fallible;
use once_cell::sync::OnceCell;
use regex::Regex;
use rustc_hash::FxHashMap;
use serde_derive::Deserialize;
use std::io::BufRead;

const URL_PREFIX: &str = "http://farm";
const URL_COMMON: &str = ".static.flickr.com/";
const URL_SUFFIX: &str = ".jpg";
const TAGS_SIZE: usize = 860621;
const GEOTAGS_SIZE: usize = 10397271;

struct GeoTag {
    time: NaiveDateTime,
    latitude: f64,
    longitude: f64,
    serv_num: char,
    url_part: String,
}

impl std::fmt::Display for GeoTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\",{},{},{}{}{}{}{}",
            self.time,
            self.latitude,
            self.longitude,
            URL_PREFIX,
            self.serv_num,
            URL_COMMON,
            self.url_part,
            URL_SUFFIX
        )
    }
}

static TAGS: OnceCell<FxHashMap<String, Vec<u64>>> = OnceCell::new();
static GEOTAGS: OnceCell<FxHashMap<u64, GeoTag>> = OnceCell::new();
static URL_REGEX: OnceCell<Regex> = OnceCell::new();
static BASE_DIR: OnceCell<String> = OnceCell::new();

fn from_str_to_geotag(s: &str) -> Fallible<(u64, GeoTag)> {
    let mut s = s.split(',');
    let id: u64 = s.next().ok_or(failure::err_msg("Id missing"))?.parse()?;
    let time: NaiveDateTime = {
        let s: &str = s.next().ok_or(failure::err_msg("Time missing"))?;
        NaiveDateTime::parse_from_str(s, "\"%Y-%m-%d %H:%M:%S\"")?
    };
    let latitude: f64 = s
        .next()
        .ok_or(failure::err_msg("Latitude missing"))?
        .parse()?;
    let longitude: f64 = s
        .next()
        .ok_or(failure::err_msg("Longitude missing"))?
        .parse()?;
    let (serv_num, url_part) = {
        let url: &str = s.next().ok_or(failure::err_msg("Url missing"))?;
        let cap = URL_REGEX
            .get()
            .unwrap()
            .captures(&url)
            .ok_or(failure::err_msg("Invalid String"))?;
        let serv_num: char = cap
            .get(1)
            .ok_or(failure::err_msg("Serv_num missing"))?
            .as_str()
            .chars()
            .nth(0)
            .ok_or(failure::err_msg("Invalid String"))?;
        let url_part: String = cap
            .get(2)
            .ok_or(failure::err_msg("Url_part missing"))?
            .as_str()
            .to_owned();
        (serv_num, url_part)
    };

    Ok((
        id,
        GeoTag {
            time,
            latitude,
            longitude,
            serv_num,
            url_part,
        },
    ))
}

fn load_tags(filename: &str) -> FxHashMap<String, Vec<u64>> {
    let f = std::fs::File::open(&format!("{}/{}", BASE_DIR.get().unwrap(), filename)).unwrap();
    let mut r = std::io::BufReader::new(f);

    let mut tags = FxHashMap::with_capacity_and_hasher(TAGS_SIZE, Default::default());
    let mut buf = String::new();
    while r.read_line(&mut buf).unwrap() != 0 {
        if buf.ends_with('\n') {
            buf.pop();
        }
        let mut s = buf.split(',');
        let id = s.next().unwrap();
        let key = s.next().unwrap();
        if !key.is_empty() {
            tags.entry(key.to_owned())
                .or_insert_with(Vec::new)
                .push(id.parse::<u64>().unwrap());
        }
        buf.clear();
    }

    for v in tags.values_mut() {
        v.sort();
    }
    tags
}

fn load_geotags(filename: &str) -> FxHashMap<u64, GeoTag> {
    let f = std::fs::File::open(&format!("{}/{}", BASE_DIR.get().unwrap(), filename)).unwrap();
    let mut r = std::io::BufReader::new(f);

    let mut geotags = FxHashMap::with_capacity_and_hasher(GEOTAGS_SIZE, Default::default());
    let mut buf = String::new();
    while r.read_line(&mut buf).unwrap() != 0 {
        if buf.ends_with('\n') {
            buf.pop();
        }
        match from_str_to_geotag(&buf) {
            Ok(i) => {
                let _ = geotags.insert(i.0, i.1);
            }
            Err(i) => println!("Ignored because {}: {}", i, buf),
        }
        buf.clear();
    }
    geotags
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct QueryWrap {
    tag: String,
}

fn query(q: web::Query<QueryWrap>) -> String {
    if let Some(i) = TAGS.get().unwrap().get(&q.tag) {
        i.iter()
            .map(|id| format!("{},{}\n", id, GEOTAGS.get().unwrap()[id]))
            .collect::<Vec<_>>()
            .join("")
    } else {
        "".into()
    }
}

fn main() {
    let _ = BASE_DIR.set(std::env::args().nth(1).unwrap());
    let _ = URL_REGEX.set(
        Regex::new(&format!(
            r"{}(\d){}(.*){}",
            URL_PREFIX, URL_COMMON, URL_SUFFIX
        ))
        .unwrap(),
    );
    let _ = TAGS.set(load_tags("tag.csv"));
    println!("Tags size : {}", TAGS.get().unwrap().len());
    let _ = GEOTAGS.set(load_geotags("geotag.csv"));
    println!("Geotags size : {}", GEOTAGS.get().unwrap().len());
    let _ = actix_web::HttpServer::new(|| {
        actix_web::App::new().service(web::resource("query.html").to(query))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}
