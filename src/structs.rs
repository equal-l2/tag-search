use serde::Deserialize;
use tag_geotag::GeoTag;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryWrap {
    pub tag: String,
}

pub struct DataPair<'a> {
    pub id: u64,
    pub geotag: &'a GeoTag,
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

pub struct CacheWrap {
    pub tag: String,
    pub content: String,
}
