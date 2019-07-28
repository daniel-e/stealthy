use std::sync::mpsc::Sender;
use crate::message::Message;
use crate::model::ItemType;
use crate::model::Item;
use crate::model::Source;

#[cfg(not(feature = "no_notify"))]
use std::process::Command;

pub enum ConsoleMessage {
    TextMessage(Item),
    Ack(u64),
    AckProgress(u64, usize, usize),
    SetScrambleTimeout(u32),
    ScrambleTick,
    Exit,
}

#[derive(Clone)]
pub struct Console {
    console: Sender<ConsoleMessage>,
}

impl Console {
    pub fn new(sender: Sender<ConsoleMessage>) -> Console {
        Console {
            console: sender
        }
    }

    pub fn new_file(&self, m: Message, filename: String) {
        new_file(self.console.clone(), m, filename);
    }

    pub fn ack_msg(&self, id: u64) {
        ack_msg(self.console.clone(), id);
    }

    pub fn ack_msg_progress(&self, id: u64, done: usize, total: usize) {
        ack_msg_progress(self.console.clone(), id, done, total);
    }

    pub fn error(&self, s: String) {
        error(self.console.clone(), s);
    }

    pub fn status(&self, s: String) {
        status(self.console.clone(), s);
    }

    pub fn new_msg(&self, m: Message) {
        new_msg(self.console.clone(), m);
    }
}

pub fn raw_item(o: Sender<ConsoleMessage>, i: Item) {
    o.send(ConsoleMessage::TextMessage(i)).expect("Error in console::msg");
}

pub fn raw(o: Sender<ConsoleMessage>, s: String, typ: ItemType, from: Source) {
    raw_item(o, Item::new(format!("{}", s), typ, from));
}

pub fn msg_item(o: Sender<ConsoleMessage>, i: Item) {
    raw_item(o, i);
}

pub fn msg(o: Sender<ConsoleMessage>, s: String, typ: ItemType, from: Source) {
    raw(o, format!("{}", s), typ, from);
}

pub fn error(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, ItemType::Error, Source::System);
}

pub fn status(o: Sender<ConsoleMessage>, s: String) {
    msg(o, s, ItemType::Info, Source::System);
}

pub fn new_file(o: Sender<ConsoleMessage>, m: Message, filename: String) {
    msg(o, format!("received file '{}'", filename), ItemType::NewFile, Source::Ip(m.get_ip()));
}

pub fn ack_msg(o: Sender<ConsoleMessage>, id: u64) {
    o.send(ConsoleMessage::Ack(id)).expect("Error");
}

pub fn ack_msg_progress(o: Sender<ConsoleMessage>, id: u64, done: usize, total: usize) {
    // TODO: "done" actually is number of pending acks
    o.send(ConsoleMessage::AckProgress(id, done, total)).expect("Error");
}

#[cfg(not(feature = "no_notify"))]
fn notify(ip: String, o: Sender<ConsoleMessage>) {
    // TODO configure the command
    if Command::new("notify-send")
        .arg("-t")
        .arg("3000")
        .arg(format!("new message from {}", ip))
        .status().is_err() {
        msg(o, format!("calling notify-send failed"), ItemType::Error, Source::System);
    }
}

pub fn new_msg(o: Sender<ConsoleMessage>, m: Message) {

    let ip = m.get_ip();
    let s  = String::from_utf8(m.get_payload());

    match s {
        Ok(s)  => {
            msg(o.clone(), format!("{}", s), ItemType::Received, Source::Ip(ip.clone()));

            #[cfg(not(feature = "no_notify"))]
            notify(ip, o);
        }
        Err(_) => {
            msg(o, format!("error: could not decode message"), ItemType::Error, Source::Ip(ip));
        }
    }
}
