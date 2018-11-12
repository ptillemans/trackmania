#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
extern crate time;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use chrono::prelude::*;
use time::Duration;
use regex::Regex;

#[derive(PartialEq, Eq, Debug)]
enum Line {
    Chat {ts: DateTime<Utc>, from: String, to: String, message: String },
    Time {ts: DateTime<Utc>, from: String, nick: String, time: Duration },
    Loading {ts: DateTime<Utc>, message: String }
}

fn parse_duration(s: &str) -> Option<Duration> {
    lazy_static! {
        static ref DURATION: Regex = Regex::new(r"(\d+):(\d{2}).(\d{2})").unwrap();
    }
    DURATION.captures(s)
        .map(|tps|
                  tps[1].parse()
                  .and_then(|m: i32|
                            tps[2].parse().map(|s: i32| 60*m + s))
                  .and_then(|s: i32|
                            tps[3].parse().map(|h: i32| 1000*s + 10*h))
             .unwrap())
        .map(|ms| Duration::milliseconds(ms as i64))
}

fn parse_line(line: String) -> Option<Line> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[([0-9 /:]*)\] (\S+) (.*)").unwrap();
        static ref RE2: Regex = Regex::new(r"\[(\S+) \((.*)\)\] ?(.*)").unwrap();
    }
    match RE.captures(&line) {
        None => None,
        Some(cap) => {
            let dt = Utc.datetime_from_str(&cap[1], "%Y/%m/%d %H:%M:%S").unwrap();
            match &cap[2] {
                "<chat>" => {
                    let cap = RE2.captures(&cap[3]).unwrap();
                    Some(Line::Chat {
                        ts: dt,
                        from: cap[1].to_string(),
                        to: cap[2].to_string(),
                        message: cap[3].to_string()
                    })},
                "<time>" => {
                    let cap = RE2.captures(&cap[3]).unwrap();
                    let duration = parse_duration(&cap[3]).unwrap();
                    Some(Line::Time {
                        ts: dt,
                        from: cap[1].to_string(),
                        nick: cap[2].to_string(),
                        time: duration
                    })
                },
                "Loading" => {
                    Some(Line::Loading {
                        ts: dt,
                        message: cap[3].to_string()
                    })
                },
                _ => None
            }
        }
    }
}

fn main() -> io::Result<()>{
    let f = File::open("GameLog.mlxadmin.txt")?;
    let f = BufReader::new(f);

    for line in f.lines()
        .map(|l| l.unwrap())
        .map(parse_line) {
        match line {
            Some(Line::Chat { ts, from, to, message}) => {
                println!("time: {}, from: {}, to: {}, msg: {}", ts, from, to, message)
            }
            Some(Line::Time { ts, from, nick: _, time}) => {
                println!("time: {}, from: {}, result: {}", ts, from, time)
            }
            Some(Line::Loading { ts, message }) => {
                println!("time: {}, loading {}", ts, message)
            }
            None => { println!("line not parsed")}
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chat_line() {
        let line = "[2017/11/24 18:54:57] <chat> [bta_ (_sof_bta_)]      wwe";
        let dt = Utc.ymd(2017, 11, 24).and_hms(18, 54, 57);
        let from = "bta_".to_string();
        let to = "_sof_bta_".to_string();
        let msg = "     wwe".to_string();

        let line = parse_line(line.to_string()).unwrap();

        assert_eq!(line, Line::Chat  {
            ts: dt,
            from: from,
            to: to,
            message: msg
        })
    }

    #[test]
    fn parse_race_line() {
        let line = "[2017/11/24 18:54:20] <time> [_iep__sej ([IEP]_sej)] 0:44.60";
        let dt = Utc.ymd(2017, 11, 24).and_hms(18, 54, 20);
        let from = "_iep__sej".to_string();
        let nick = "[IEP]_sej".to_string();
        let duration = Duration::milliseconds(44600);

        let line = parse_line(line.to_string()).unwrap();

        assert_eq!(line, Line::Time  {
            ts: dt,
            from: from,
            nick: nick,
            time: duration
        })
    }

    #[test]
    fn parse_race_line_with_smiley() {
        let line = "[2017/11/24 21:26:00] <time> [sop_1912 ([ERF] SOP :-))] 0:23.75";
        let dt = Utc.ymd(2017, 11, 24).and_hms(21, 26, 00);
        let from = "sop_1912".to_string();
        let nick = "[ERF] SOP :-)".to_string();
        let duration = Duration::milliseconds(23750);

        let line = parse_line(line.to_string()).unwrap();

        assert_eq!(line, Line::Time  {
            ts: dt,
            from: from,
            nick: nick,
            time: duration
        })
    }

    #[test]
    fn parse_empty_chat_line_() {
        let line = "[2017/11/24 22:06:58] <chat> [ieper_mlj (ieper_mlj)]";
        let dt = Utc.ymd(2017, 11, 24).and_hms(22, 06, 58);
        let from = "ieper_mlj".to_string();
        let to = "ieper_mlj".to_string();

        let line = parse_line(line.to_string()).unwrap();

        assert_eq!(line, Line::Chat {
            ts: dt,
            from: from,
            to: to,
            message: "".to_string()
        })
    }

    #[test]
    fn parse_loading_line() {
        let line = "[2017/11/24 22:52:47] Loading challenge 12.Gbx (VwweH5HaaHGfddasysZI03s4Wy9)...";
        let dt = Utc.ymd(2017, 11, 24).and_hms(22, 52, 47);
        let msg = "challenge 12.Gbx (VwweH5HaaHGfddasysZI03s4Wy9)...".to_string();

        let line = parse_line(line.to_string()).unwrap();

        assert_eq!(line, Line::Loading {
            ts: dt,
            message: msg
        })
    }
}
