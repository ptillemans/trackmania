extern crate regex;
extern crate chrono;
extern crate time;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use chrono::prelude::*;
use time::Duration;

enum Line {
    Chat {ts: DateTime<Utc>, from: String, to: String, message: String },
    Time {ts: DateTime<Utc>, from: String, nick: String, time: Duration }
}

fn parse_line(line: String) -> Line {
    Line::Chat {
        ts: Utc::now(), 
        from: "test".to_string(),
        to: "test".to_string(), 
        message: "test".to_string()}
}

fn main() -> io::Result<()>{
    let f = File::open("GameLog.mlxadmin.txt")?;
    let f = BufReader::new(f);

    for line in f.lines()
        .map(|l| l.unwrap())
        .map(parse_line) {
        match line {
            Line::Chat { ts, from, to, message} => {
                println!("time: {}, from: {}, to: {}, msg: {}", ts, from, to, message)
            }
            Line::Time { ts, from, nick: _, time} => {
                println!("time: {}, from: {}, result: {}", ts, from, time)
            }
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    #[test]
    fn parse_chat_line() {
        assert_eq!(2 + 2, 4);
    }
}