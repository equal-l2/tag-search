use once_cell::sync::OnceCell;
use std::io::BufRead;
use tag_geotag::*;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

pub const TAGS_SIZE: usize = 860621;
pub const GEOTAGS_SIZE: usize = 6145483;
pub static BASE_DIR: OnceCell<String> = OnceCell::new();

pub fn load_tags(filename: &str) -> HashMap<String, Vec<u64>> {
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

pub fn load_geotags(filename: &str) -> HashMap<u64, GeoTag> {
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
