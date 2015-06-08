#[cfg(not(feature="usencurses"))]
extern crate term;

#[cfg(not(feature="usencurses"))]
use std::io;
#[cfg(not(feature="usencurses"))]
use term::color;

#[cfg(not(feature="usencurses"))]
use humaninterface::{Input, Output};
#[cfg(not(feature="usencurses"))]
use callbacks::Callbacks;

#[cfg(not(feature="usencurses"))]
pub struct StdIn;
#[cfg(not(feature="usencurses"))]
pub struct StdOut;

#[cfg(not(feature="usencurses"))]
impl StdOut {

    pub fn new() -> StdOut {
        StdOut
    }
}

#[cfg(not(feature="usencurses"))]
impl StdIn {

    pub fn new() -> StdIn {
        StdIn
    }
}

#[cfg(not(feature="usencurses"))]
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

#[cfg(not(feature="usencurses"))]
impl Output for StdOut {

    fn close(&self) { }

    fn println(&mut self, s: String, color: color::Color) {
        println_colored(s, color);
    }
}

#[cfg(not(feature="usencurses"))]
impl Callbacks for StdOut { } // use default implementations


#[cfg(not(feature="usencurses"))]
pub fn println_colored(msg: String, color: term::color::Color) {

    let mut t = term::stdout().unwrap();
    t.fg(color).unwrap();
    (write!(t, "{}", msg)).unwrap();
    t.reset().unwrap();
    (write!(t, "\n")).unwrap();
}


