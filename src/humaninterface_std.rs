extern crate term;

use std::io;
use term::color;

use humaninterface::{Input, Output};
use callbacks::Callbacks;
use tools::println_colored;

pub struct StdIn;
pub struct StdOut;

impl StdOut {

    pub fn new() -> StdOut {
        StdOut
    }
}

impl StdIn {

    pub fn new() -> StdIn {
        StdIn
    }
}

impl Input for StdIn {

    fn read_line(&self) -> Option<String> {
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(n) => {
                if n != 0 { Some(s) } else { None }
            }
            _ => None
        }
    }
}

impl Output for StdOut {

    fn close(&self) { }

    fn println(&mut self, s: String, color: color::Color) {
        println_colored(s, color);
    }
}

impl Callbacks for StdOut { } // use default implementations


