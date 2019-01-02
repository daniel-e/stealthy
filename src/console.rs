use std::sync::mpsc::Sender;
use stealthy::Message;
use crate::model::ItemType;
use crate::model::Item;

#[cfg(not(feature = "no_notify"))]
use std::process::Command;

pub enum ConsoleMessage {
    TextMessage(Item),
    Ack(u64),
    Exit,
}

fn fm_time() -> String {
    time::strftime("%d.%m. %R", &time::now()).unwrap()
}

pub fn raw_item(o: Sender<ConsoleMessage>, i: Item) {
    o.send(ConsoleMessage::TextMessage(i)).expect("Error in console::msg");
}

pub fn raw(o: Sender<ConsoleMessage>, s: String, typ: ItemType) {
    raw_item(o, Item::new(format!("{}", s), typ));
}

pub fn msg_item(o: Sender<ConsoleMessage>, i: Item) {
    let s = i.msg.clone();
    raw_item(o, i.message(format!("{} │ {}", fm_time(), s)));
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

pub fn ack_msg(o: Sender<ConsoleMessage>, id: u64) {
    o.send(ConsoleMessage::Ack(id)).expect("Error");
}

#[cfg(not(feature = "no_notify"))]
fn notify(ip: String, o: Sender<ConsoleMessage>) {
    // TODO configure the command
    if Command::new("notify-send")
        .arg("-t")
        .arg("3000")
        .arg(format!("new message from {}", ip))
        .status().is_err() {
        msg(o, format!("calling notify-send failed"), ItemType::Error);
    }
}

pub fn new_msg(o: Sender<ConsoleMessage>, m: Message) {

    let ip = m.get_ip();
    let s  = String::from_utf8(m.get_payload());

    match s {
        Ok(s)  => {
            msg(o.clone(), format!("[{}] {}", ip, s), ItemType::Received);

            #[cfg(not(feature = "no_notify"))]
            notify(ip, o);
        }
        Err(_) => {
            msg(o, format!("[{}] error: could not decode message", ip), ItemType::Error);
        }
    }
}
