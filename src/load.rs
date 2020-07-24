use std::io::BufRead;
use std::path::Path;
use tag_geotag::*;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

// # of entries in TagsTable
pub const TAGS_SIZE: usize = 860621;

// # of entries in GeoTagsTable
pub const GEOTAGS_SIZE: usize = 6145483;

// # of entries to show in response
pub const ENTRY_COUNT: usize = 100;

pub type TagsTable = HashMap<String, Vec<u64>>;
pub type GeoTagsTable = HashMap<u64, GeoTag>;

pub fn load_tags(path: &Path, geotags: &GeoTagsTable) -> TagsTable {
    let f = std::fs::File::open(path).unwrap();
    let r = std::io::BufReader::new(f);

    let mut tags = TagsTable::with_capacity_and_hasher(TAGS_SIZE, Default::default());
    // Note that tag_pp.csv has "NO_TAG" at the first line
    for s in r.lines().skip(1) {
        let mut s = s.unwrap();
        if s.ends_with('\n') {
            s.pop();
        }
        let mut sp = s.split(',');
        let key = sp.next().unwrap();
        // skip the size column
        let ids = {
            let mut raw = sp.skip(1).map(|s| s.parse().unwrap()).collect::<Vec<_>>();
            raw.sort_by(|a, b| geotags[a].time.cmp(&geotags[b].time).reverse());
            raw.into_iter().take(ENTRY_COUNT).collect()
        };

        tags.insert(key.to_owned(), ids);
    }

    tags
}

pub fn load_geotags(path: &Path) -> GeoTagsTable {
    let mut geotags = GeoTagsTable::with_capacity_and_hasher(GEOTAGS_SIZE, Default::default());

    let f = std::fs::File::open(path).unwrap();
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
