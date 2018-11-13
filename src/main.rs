#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate time;

use std::fs::File;
use std::collections::HashMap;
use std::cmp::min;
use std::io::prelude::*;
use std::io::{self, BufReader};
use time::Duration;

mod parser;

use parser::{parse_line, Line};

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
