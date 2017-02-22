use frontend::term::color;

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

    fn scroll_up(&mut self) { }
    fn scroll_down(&mut self) { }
}
