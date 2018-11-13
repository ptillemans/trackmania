#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate time;

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};

mod parser;

use parser::{parse_line, Line};

fn main() -> io::Result<()> {
    let f = File::open("GameLog.mlxadmin.txt")?;
    let f = BufReader::new(f);

    for line in f.lines().map(|l| l.unwrap()).map(parse_line) {
        match line {
            Some(Line::Chat {
                ts,
                from,
                to,
                message,
            }) => println!("time: {}, from: {}, to: {}, msg: {}", ts, from, to, message),
            Some(Line::Time {
                ts,
                from,
                nick: _,
                time,
            }) => println!("time: {}, from: {}, result: {}", ts, from, time),
            Some(Line::Loading { ts, message }) => println!("time: {}, loading {}", ts, message),
            None => println!("line not parsed"),
        }
    }

    Ok(())
}
