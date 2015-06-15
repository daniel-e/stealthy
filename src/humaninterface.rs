extern crate term;

use term::color;

pub enum ControlType {
    ArrowUp,
    ArrowDown
}

pub enum UserInput {
    Line(String),
    Control(ControlType)
}

pub trait Input {
    fn read_line(&self) -> Option<UserInput>;
}

pub trait Output {
    fn close(&self);
    fn println(&mut self, s: String, color: color::Color);
}

