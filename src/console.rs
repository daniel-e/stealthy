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

pub enum ConsoleMessage {
    TextMessage(NormalMessage),
    Exit,
    ScrollUp,
    ScrollDown,
    Refresh,
}

pub struct NormalMessage {
    pub msg: String,
    pub col: Color
}

impl NormalMessage {
    pub fn new(msg: String, col: Color) -> NormalMessage {
        NormalMessage {
            msg: msg,
            col: col,
        }
    }
}

fn fm_time() -> String {
    time::strftime("%d.%m. %R", &time::now()).unwrap()
}

pub fn raw(o: Sender<ConsoleMessage>, s: String, col: Color) {
    o.send(ConsoleMessage::TextMessage(
        NormalMessage::new(format!("{}", s), col))
    ).expect("Error in console::msg");
}

pub fn msg(o: Sender<ConsoleMessage>, s: String, col: Color) {
    raw(o, format!("{} │ {}", fm_time(), s), col);
}

pub fn error(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, Color::BrightRed);
}

pub fn status(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, Color::BrightYellow);
}

pub fn new_file(o: Sender<ConsoleMessage>, m: Message, filename: String) {
    msg(o, format!("[{}] received file '{}'", m.get_ip(), filename), Color::BrightGreen);
}

pub fn ack_msg(o: Sender<ConsoleMessage>, _id: u64) {
    msg(o, format!("ack"), Color::BrightGreen);
}

pub fn new_msg(o: Sender<ConsoleMessage>, m: Message) {

    let ip = m.get_ip();
    let s  = String::from_utf8(m.get_payload());

    match s {
        Ok(s)  => {
            msg(o.clone(), format!("[{}] {}", ip, s), Color::Yellow);

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
            msg(o, format!("[{}] error: could not decode message", ip), Color::BrightRed);
        }
    }
}
