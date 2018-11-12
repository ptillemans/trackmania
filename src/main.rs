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
    Time {ts: DateTime<Utc>, from: String, nick: String, time: Duration }
}

fn parse_line(line: String) -> Option<Line> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[([0-9 /:]*)\].* <(\w+)> \[(\S+) \((\S+)\)] (.*)").unwrap();
        static ref DURATION: Regex = Regex::new(r"(\d+):(\d{2}).(\d{2})").unwrap();
    }
    match RE.captures(&line) {
        None => None,
        Some(cap) => {
            let dt = Utc.datetime_from_str(&cap[1], "%Y/%m/%d %H:%M:%S").unwrap();
            match &cap[2] {
                "chat" => Some(Line::Chat {
                        ts: dt,
                        from: cap[3].to_string(),
                        to: cap[4].to_string(),
                        message: cap[5].to_string()
                    }),
                "time" => {
                    let tps = DURATION.captures(&cap[5]).unwrap();
                    let min: i32 = tps[1].parse().unwrap();
                    let sec: i32 = tps[2].parse().unwrap();
                    let huns: i32 = tps[3].parse().unwrap();
                    let ms = 10 * huns + 1000 * (60 * min + sec);
                    let duration = Duration::milliseconds(ms as i64);
                    Some(Line::Time {
                        ts: dt,
                        from: cap[3].to_string(),
                        nick: cap[4].to_string(),
                        time: duration
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

}
