extern crate term;

use term::color;

pub trait InputOutput {
    fn println(&self, s: String, color: color::Color);
    fn read_line(&self) -> Option<String>;
}

