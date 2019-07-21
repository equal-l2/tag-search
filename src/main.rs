use regex::Regex;
use std::io::BufRead;
use std::io::Write;

const BASEDIR: &str = "/home/pi/last-data";

type HashSet<K> = rustc_hash::FxHashSet<K>;

fn main() {
    let tag = std::env::args().nth(1);
    if tag.is_none() {
        std::process::exit(1);
    }
    let tag = tag.unwrap();

    let tag_re = Regex::new(&format!(r"^(\d{{0,10}}),{}$", tag)).unwrap();
    let f = std::fs::File::open(&format!("{}/tag.csv", BASEDIR)).unwrap();
    let r = std::io::BufReader::new(f);

    let mut ids: HashSet<u64> = r
        .lines()
        .filter_map(|s| {
            let mut s = s.unwrap();
            if s.ends_with('\n') {
                s.pop();
            }
            if let Some(i) = tag_re.captures(&s) {
                i.get(1).unwrap().as_str().parse().ok()
            } else {
                None
            }
        })
        .collect();

    if ids.is_empty() {
        return;
    }

    eprintln!("{} ids found", ids.len());

    let f = std::fs::File::open(&format!("{}/geotag.csv", BASEDIR)).unwrap();
    let r = std::io::BufReader::new(f);

    let stdout = std::io::stdout();
    let mut w = stdout.lock();

    for s in r.lines() {
        let s = s.unwrap();
        let id = s.split(',').nth(0).unwrap().parse();
        if let Ok(i) = id {
            if ids.contains(&i) {
                write!(w, "{}\n", s);
                ids.remove(&i);
                if ids.is_empty() {
                    break;
                }
            }
        }
    }
}
