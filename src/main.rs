mod load;

use actix_web::{web, HttpResponse};
use once_cell::sync::OnceCell;
use std::fmt::Write;

use tag_geotag::*;

#[cfg(feature = "cache")]
mod cache;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

static TAGS: OnceCell<load::TagsTable> = OnceCell::new();
static GEOTAGS: OnceCell<load::GeoTagsTable> = OnceCell::new();

#[derive(serde::Deserialize, Debug)]
struct QueryWrap {
    tag: String,
    #[cfg(feature = "cache")]
    cache: Option<bool>,
}

pub struct DataPair<'a> {
    pub id: u64,
    pub geotag: &'a GeoTag,
}

fn generate_html_from_iter<'a, I>(data: I) -> String
where
    I: Iterator<Item = DataPair<'a>>,
{
    let mut s = String::with_capacity(18600);
    s.push_str(r#"<!doctype html><html><head><title>超高性能化</title><meta charset="utf-8"></head><body>"#);
    for d in data {
        let datetime = time::OffsetDateTime::from_unix_timestamp(d.geotag.time as i64)
            .expect("timestamp out of range");
        let _ = write!(
            s,
            "<div><img src={} alt={}><p>Latitude : {}<br>Longitude : {}<br>Shot at {}-{:02}-{:02} {:02}:{:02}:{:02}</p></div>",
            d.geotag.get_url(d.id),
            d.id,
            d.geotag.latitude,
            d.geotag.longitude,
            datetime.year(),
            datetime.month() as u8,
            datetime.day(),
            datetime.hour(),
            datetime.minute(),
            datetime.second(),
        );
    }
    s.push_str("</body></html>");
    s
}

// SAFETY: tag always exists
#[cfg(feature = "cache")]
fn handle_cache(tag: &str) -> HttpResponse {
    let mut response = HttpResponse::Ok();
    response.content_type("text/html");

    let cont = &cache::CACHE;
    loop {
        if let Some(i) = cont.read().get(tag) {
            return response.body(i);
        }
        if let Some(mut lock) = cont.try_write() {
            let s = generate(tag);
            let res = response.body(&s);
            lock.push(tag.to_owned(), s);
            return res;
        }
    }
}

// SAFETY: tag always exists
fn generate(tag: &str) -> String {
    let (tags, geotags) = unsafe {
        // SAFETY: this function is never called before the server is launched
        (TAGS.get_unchecked(), GEOTAGS.get_unchecked())
    };
    let pairs = tags.get(tag).unwrap().iter().map(|id| DataPair {
        id: *id,
        geotag: &geotags[&id],
    });
    generate_html_from_iter(pairs)
}

async fn query(q: web::Query<QueryWrap>) -> HttpResponse {
    let tags = unsafe {
        // SAFETY: this function is never called before the server is launched
        TAGS.get_unchecked()
    };

    if tags.contains_key(&q.tag) {
        #[cfg(feature = "cache")]
        {
            let use_cache = q.cache.unwrap_or(true);
            if use_cache {
                return handle_cache(&q.tag);
            }
        }

        let mut response = HttpResponse::Ok();
        response.content_type("text/html").body(generate(&q.tag))
    } else {
        let mut response = HttpResponse::Ok();
        response.content_type("text/html").body(r#"<!doctype html><html><head><title>超高性能化</title><meta charset="utf-8"></head><body><p>No Match</p></body></html>"#)
    }
}

#[actix_web::main]
async fn main() {
    // parse args
    let mut args = std::env::args().skip(1);
    let base_dir =
        std::path::PathBuf::from(args.next().expect("Base directory is required as 1st arg"));

    // initialize global variables
    println!("Now loading... (Wait patiently)");
    let now = std::time::Instant::now();

    let _ = GEOTAGS.set(load::load_geotags(&base_dir.join("geotag_pp.csv")));
    println!("Geotags size : {}", GEOTAGS.get().unwrap().len());
    let _ = TAGS.set(load::load_tags(
        &base_dir.join("tag_pp.csv"),
        GEOTAGS.get().unwrap(),
    ));
    println!("Tags size : {}", TAGS.get().unwrap().len());

    println!(
        "Load complete, elapsed time : {}[s]",
        (now.elapsed().as_millis() as f64) / 1000f64
    );

    #[cfg(feature = "cache")]
    {
        cache::init();
        println!("Cache initialized");
    }

    // start the web server
    let bind_addr = "0.0.0.0:8080";
    println!("Server listening at '{}'", bind_addr);
    let _ = actix_web::HttpServer::new(|| {
        actix_web::App::new().service(web::resource("query.html").to(query))
    })
    .bind(bind_addr)
    .unwrap()
    .run()
    .await;
}
