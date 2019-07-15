use regex::Regex;
use std::io::BufRead;

const BASEDIR: &str = "/home/pi/last-data";

fn main() {
    let mut ids: Vec<String> = vec![];
    {
        let tag = std::env::args().nth(1);
        if tag.is_none() {
            std::process::exit(1);
        }
        let tag = tag.unwrap();

        let tag_re = Regex::new(&format!(r"^(\d+),{}$", tag)).unwrap();

        let f = std::fs::File::open(&format!("{}/tag.csv", BASEDIR)).unwrap();
        let mut r = std::io::BufReader::new(f);

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
    }

    //eprintln!("{} ids found", ids.len());

    {
        let mut res: Vec<_> = ids.iter().map(|s| Regex::new(&format!(r"^{},", s)).unwrap()).collect();

        let f = std::fs::File::open(&format!("{}/geotag.csv", BASEDIR)).unwrap();
        let mut r = std::io::BufReader::new(f);

        let mut buf = String::new();

        while r.read_line(&mut buf).unwrap() != 0 && !res.is_empty() {
            if buf.ends_with('\n') {
                buf.pop();
            }

            let mut matched = None;
            for (i, re) in res.iter().enumerate() {
                if re.is_match(&buf) {
                    println!("{}", buf);
                    matched = Some(i);
                    break;
                }
            }

            if let Some(i) = matched {
                //eprintln!("{} removed", res[i]);
                res.remove(i);
            }

            buf.clear();
        }
    }

}
