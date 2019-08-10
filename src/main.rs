use std::io::BufRead;
use std::io::Write;

type HashSet<K> = rustc_hash::FxHashSet<K>;

fn main() {
    let mut args = std::env::args().skip(1);
    let basedir = args.next().unwrap();
    let tag = args.next().unwrap();
    if tag.is_empty() {
        return;
    }

    let f = std::fs::File::open(&format!("{}/tag.csv", basedir)).unwrap();
    let r = std::io::BufReader::new(f);

    let mut ids: HashSet<u64> = r
        .lines()
        .filter_map(|s| {
            let mut s = s.unwrap();
            if s.ends_with('\n') {
                s.pop();
            }
            let mut sp = s.split(',');
            let id_str = sp.next().unwrap();
            let tag_str = sp.next().unwrap();
            if tag == tag_str {
                id_str.parse().ok()
            } else {
                None
            }
        })
        .collect();

    if ids.is_empty() {
        return;
    }

    eprintln!("{} ids found", ids.len());

    let f = std::fs::File::open(&format!("{}/geotag.csv", basedir)).unwrap();
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
