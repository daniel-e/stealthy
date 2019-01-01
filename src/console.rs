use std::sync::mpsc::Sender;
use std::process::Command;
use stealthy::Message;
use crate::ui_termion::ItemType;

pub enum ConsoleMessage {
    TextMessage(NormalMessage),
    Exit,
    ScrollUp,
    ScrollDown,
    Refresh,
}

pub struct NormalMessage {
    pub msg: String,
    pub typ: ItemType
}

impl NormalMessage {
    pub fn new(msg: String, typ: ItemType) -> NormalMessage {
        NormalMessage {
            msg,
            typ
        }
    }
}

fn fm_time() -> String {
    time::strftime("%d.%m. %R", &time::now()).unwrap()
}

pub fn raw(o: Sender<ConsoleMessage>, s: String, typ: ItemType) {
    o.send(ConsoleMessage::TextMessage(
        NormalMessage::new(format!("{}", s), typ))
    ).expect("Error in console::msg");
}

pub fn msg(o: Sender<ConsoleMessage>, s: String, typ: ItemType) {
    raw(o, format!("{} │ {}", fm_time(), s), typ);
}

pub fn error(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, ItemType::Error);
}

pub fn status(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, ItemType::Info);
}

pub fn new_file(o: Sender<ConsoleMessage>, m: Message, filename: String) {
    msg(o, format!("[{}] received file '{}'", m.get_ip(), filename), ItemType::NewFile);
}

pub fn ack_msg(o: Sender<ConsoleMessage>, _id: u64) {
    msg(o, format!("ack"), ItemType::Ack);
}

pub fn new_msg(o: Sender<ConsoleMessage>, m: Message) {

    let ip = m.get_ip();
    let s  = String::from_utf8(m.get_payload());

    match s {
        Ok(s)  => {
            msg(o.clone(), format!("[{}] {}", ip, s), ItemType::Received);

            // TODO configure the command
            if Command::new("notify-send")
                .arg("-t")
                .arg("3000")
                .arg(format!("new message from {}", ip))
                .status().is_err() {
                msg(o, format!("calling notify-send failed"), ItemType::Error);
            }
        }
        Err(_) => {
            msg(o, format!("[{}] error: could not decode message", ip), ItemType::Error);
        }
    }
}
