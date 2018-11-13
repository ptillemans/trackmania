#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
extern crate time;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use std::cmp::min;

use chrono::prelude::*;
use time::Duration;
use regex::Regex;

#[derive(PartialEq, Eq, Debug)]
enum Line {
    Chat {ts: DateTime<Utc>, from: String, to: String, message: String },
    Time {ts: DateTime<Utc>, from: String, nick: String, time: Duration },
    Loading {ts: DateTime<Utc>, track: String }
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
                        track: cap[3].to_string()
                    })
                },
                _ => None
            }
        }
    }
}

struct TrackResult {
    best_results: HashMap<String, Duration>
}

impl TrackResult {
    fn new() -> TrackResult {
        TrackResult {
            best_results: HashMap::new()
        }
    }

    fn update_player(&mut self, nick: String, time: Duration) {
        let best_result = self.best_results.entry(nick).or_insert(time);
        *best_result = min(*best_result, time);
    }
}

struct State {
    track: String,
    results: HashMap<String, TrackResult>
}

impl State {
    fn new() -> State {
        State { 
            track: String::from("initial"),
            results: HashMap::new()
        }
    }

    fn add_line(mut self: State, line: Line) -> State {
        match line {
            Line::Chat {..} => self,
            Line::Time {ts: _, from: _, nick, time} => {
                let track = self.track.clone();
                self.results.entry(track)
                .and_modify(|ref mut result| result.update_player(nick, time))
                .or_insert_with(|| TrackResult::new());
                self
            },
            Line::Loading { ts: _, track } => {
                println!("switch track to {}", track);
                self.track = track;
                self
            }
        }
    }
}

fn main() -> io::Result<()>{
    let f = File::open("GameLog.mlxadmin.txt")?;
    let f = BufReader::new(f);

    let begin = State::new();
    let _end = f.lines()
        .map(|l| l.unwrap())
        .filter_map(parse_line)
        .fold(begin, State::add_line);

    for (track, _results) in _end.results {
        println!("Track {}", track);
        let mut sorted_scores: Vec<(Duration, String)> = _results.best_results
            .iter()
            .map(|(nick, time)| (*time, nick.clone()))
            .collect();
        sorted_scores.sort();

        
        for (best_time, nick) in sorted_scores {
            println!("  nick {} : {}", nick, best_time);
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
            track: msg
        })
    }
}
