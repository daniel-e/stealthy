extern crate term;

use std::io;
use term::color;

use humaninterface::InputOutput;
use callbacks::Callbacks;
use tools::println_colored;

pub struct Std;

impl Std {

    pub fn new() -> Std {
        Std
    }
}

impl InputOutput for Std {

    fn println(&self, s: String, color: color::Color) {
        println_colored(s, color);
    }

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

impl Callbacks for Std { }


