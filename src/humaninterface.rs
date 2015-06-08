extern crate term;

use term::color;

pub trait Input {
    fn read_line(&self) -> Option<String>;
}

pub trait Output {
    fn close(&self);
    fn println(&self, s: String, color: color::Color);
}

