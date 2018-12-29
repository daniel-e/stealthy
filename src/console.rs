use term::color;
use std::sync::mpsc::Sender;
use std::process::Command;
use stealthy::Message;

pub enum Color {
    BrightGreen,
    White,
    Yellow,
    Blue,
    Red,
    Green,
    BrightRed,
    BrightYellow,
}

fn map_color(c: Color) -> color::Color {
    match c {
        Color::BrightGreen => color::BRIGHT_GREEN,
        Color::White => color::WHITE,
        Color::Yellow => color::YELLOW,
        Color::Blue => color::BLUE,
        Color::Red => color::RED,
        Color::Green => color::GREEN,
        Color::BrightRed => color::BRIGHT_RED,
        Color::BrightYellow => color::BRIGHT_YELLOW
    }
}


pub enum ConsoleMessage {
    TextMessage(NormalMessage),
    Exit,
    ScrollUp,
    ScrollDown,
}

pub struct NormalMessage {
    pub msg: String,
    pub col: color::Color
}

impl NormalMessage {
    pub fn new(msg: String, col: color::Color) -> NormalMessage {
        NormalMessage {
            msg: msg,
            col: col,
        }
    }
}


pub fn msg(o: Sender<ConsoleMessage>, s: String, col: Color) {
    o.send(ConsoleMessage::TextMessage(
        NormalMessage::new(s, map_color(col)))).expect("Error in console::msg");
}

pub fn error(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, Color::BrightRed);
}

pub fn status(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, Color::BrightYellow);
}

pub fn new_file(o: Sender<ConsoleMessage>, m: Message, filename: String) {
    let fm = time::strftime("%R", &time::now()).unwrap();
    msg(o, format!("{} [{}] received file '{}'", fm, m.get_ip(), filename), Color::BrightGreen);
}

pub fn ack_msg(o: Sender<ConsoleMessage>, _id: u64) {
    let fm = time::strftime("%R", &time::now()).unwrap();
    msg(o, format!("{} ack", fm), Color::BrightGreen);
}

pub fn new_msg(o: Sender<ConsoleMessage>, m: Message) {

    let ip = m.get_ip();
    let s  = String::from_utf8(m.get_payload());
    let fm = time::strftime("%R", &time::now()).unwrap();

    match s {
        Ok(s)  => {
            msg(o.clone(), format!("{} [{}] says: {}", fm, ip, s), Color::Yellow);

            // TODO configure the command
            if Command::new("notify-send")
                .arg("-t")
                .arg("3000")
                .arg(format!("new message from {}", ip))
                .status().is_err() {
                msg(o, format!("calling notify-send failed"), Color::Red);
            }
        }
        Err(_) => {
            msg(o, format!("[{}] {} error: could not decode message", ip, fm), Color::BrightRed);
        }
    }
}
