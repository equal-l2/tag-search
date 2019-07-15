use regex::Regex;
use std::io::BufRead;
use std::sync::{Arc, RwLock};
use std::thread;

const BASEDIR: &str = "/home/pi/last-data/split";

fn get_id(filename: &str, tag_re: &Arc<Regex>) -> Vec<String> {
    let f = std::fs::File::open(&format!("{}/{}", BASEDIR, filename)).unwrap();
    let mut r = std::io::BufReader::new(f);

    let mut ids: Vec<String> = vec![];
    let mut buf = String::new();

    while r.read_line(&mut buf).unwrap() != 0 {
        if buf.ends_with('\n') {
            buf.pop();
        }
        if let Some(i) = tag_re.captures(&buf) {
            ids.push(i.get(1).unwrap().as_str().to_owned());
        }
        buf.clear();
    }
    ids
}

fn get_info(filename: &str, res: &Arc<RwLock<Vec<Regex>>>) -> Vec<String> {
    let f = std::fs::File::open(&format!("{}/{}", BASEDIR, filename)).unwrap();
    let mut r = std::io::BufReader::new(f);

    let mut infos: Vec<String> = vec![];
    let mut buf = String::new();

    while r.read_line(&mut buf).unwrap() != 0 && !res.read().unwrap().is_empty() {
        if buf.ends_with('\n') {
            buf.pop();
        }

        let mut matched = None;
        for (i, re) in res.read().unwrap().iter().enumerate() {
            if re.is_match(&buf) {
                infos.push(buf.clone());
                matched = Some(i);
                break;
            }
        }

        if let Some(i) = matched {
            res.write().unwrap().remove(i);
        }

        buf.clear();
    }
    infos
}

fn main() {
    let mut ids: Vec<String> = vec![];
    {
        let tag = std::env::args().nth(1);
        if tag.is_none() {
            std::process::exit(1);
        }
        let tag = tag.unwrap();
        let tag_re = std::sync::Arc::new(Regex::new(&format!(r"^(\d+),{}$", tag)).unwrap());

        let handles: Vec<_> = ["tagaa", "tagab", "tagac", "tagad", "tagae"]
            .iter()
            .map(|s| {
                let tag_re_cp = tag_re.clone();
                thread::spawn(move || get_id(&s, &tag_re_cp))
            })
            .collect();

        for h in handles {
            ids.extend_from_slice(&h.join().unwrap());
        }
    }

    //eprintln!("{} ids found", ids.len());

    {
        let res: Arc<RwLock<Vec<_>>> = Arc::new(RwLock::new(
            ids.iter()
                .map(|s| Regex::new(&format!(r"^{},", s)).unwrap())
                .collect(),
        ));

        let handles: Vec<_> = ["geotagaa", "geotagab", "geotagac", "geotagad", "geotagae"]
            .iter()
            .map(|s| {
                let res_cp = res.clone();
                thread::spawn(move || get_info(&s, &res_cp))
            })
            .collect();

        let mut outs: Vec<String> = vec![];
        for h in handles {
            outs.extend_from_slice(&h.join().unwrap());
        }

        for s in outs {
            println!("{}", s);
        }
    }
}
