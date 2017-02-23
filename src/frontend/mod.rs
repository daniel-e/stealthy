extern crate term;
extern crate time;

use self::term::color;

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub mod humaninterface;
mod humaninterface_ncurses;
mod humaninterface_std;
mod callbacks;
mod globalstate;

use frontend::humaninterface::{Output, Input, UserInput, ControlType};
use frontend::callbacks::Callbacks;
use frontend::globalstate::GlobalState;

// TODO refactor those dependencies
use ::misc::IncomingMessage;
use ::misc::Message;
use ::misc::Layers;
use ::misc::Errors;
use super::tools::read_file;

#[cfg(not(feature="usencurses"))]
use humaninterface_std::{StdIn, StdOut};
#[cfg(not(feature="usencurses"))]
type HiIn = StdIn;
#[cfg(not(feature="usencurses"))]
type HiOut = StdOut;

#[cfg(feature="usencurses")]
use frontend::humaninterface_ncurses::{NcursesIn, NcursesOut};
#[cfg(feature="usencurses")]
type HiIn = NcursesIn;
#[cfg(feature="usencurses")]
type HiOut = NcursesOut;

pub use self::color::WHITE;
pub use self::color::GREEN;
pub use self::color::YELLOW;

pub struct Gui {
    pub o: Arc<Mutex<HiOut>>,
    i: HiIn,
    state: GlobalState,
}

impl Gui {
    pub fn new() -> Gui {
        let o = Arc::new(Mutex::new(HiOut::new()));
        Gui {
            o: o.clone(), // interface for output
            i: HiIn::new(), // interface for input
            state: GlobalState::new(),
        }
    }

    pub fn println(&self, s: String, c: color::Color) {
        let mut out = self.o.lock().unwrap();
        out.println(s, c);
    }

    pub fn input_loop(&self, l: Layers, dstip: String) {

        // read from human interface until user enters control-d and send the
        // message via the network layer
        loop { match self.i.read_line() {
                Some(ui) => {
                    match ui {
                        UserInput::Line(s) => {
                            let txt = s.trim_right().to_string();
                            if txt.len() > 0 {
                                if txt.starts_with("/") {
                                    parse_command(txt, self.o.clone(), &l, dstip.clone(), &self.state);
                                } else {
                                    send_message(txt, self.o.clone(), &l, dstip.clone());
                                }
                            }
                        }
                        UserInput::Control(what) => {
                            let mut out = self.o.lock().unwrap();
                            match what {
                                ControlType::ArrowUp => out.scroll_up(),
                                ControlType::ArrowDown => out.scroll_down()
                            }
                        }
                    }
                }
                _ => { break; }
        }}
        self.o.lock().unwrap().close();
    }

    pub fn get_channel(&self) -> Sender<IncomingMessage> {
        let (tx, rx) = channel();
        let o = self.o.clone();
        thread::spawn(move || {
            loop { match rx.recv() {
                Ok(msg) => {
                    let mut out = o.lock().unwrap();
                    match msg {
                        IncomingMessage::New(msg) =>    { out.new_msg(msg); }
                        IncomingMessage::Ack(id)  =>    { out.ack_msg(id); }
                        IncomingMessage::Error(_, s) => { out.err_msg(s); }
                    }
                }
                Err(e) => {
                    let mut out = o.lock().unwrap();
                    out.println(format!("recv_loop: failed to receive message. {:?}", e), color::RED);
                }
            }
        }});
        tx
    }
}

pub fn help_message(o: Arc<Mutex<HiOut>>) {

    let lines = vec![
        "Commands always start with a slash:",
        "/help           - this help message",
        "arrow up        - scroll to older messages",
        "arrow down      - scroll to latest messages",
        "/uptime, /up    - uptime",
        "/cat <filename> - send content of an UTF-8 encoded text file"
    ];

    for v in lines {
        output(v, o.clone());
    }
}

fn output(msg: &str, o: Arc<Mutex<HiOut>>) {

    o.lock().unwrap().println(String::from(msg), color::WHITE);
}

pub fn parse_command(txt: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String, state: &GlobalState) {
    // TODO: find more elegant solution for this
    if txt.starts_with("/cat ") {
        // TODO split_at works on bytes not characters
        let (_, b) = txt.as_str().split_at(5);
        match read_file(b) {
            Ok(data) => {
                o.lock().unwrap().
                    println(String::from("Transmitting data ..."), color::WHITE);
                send_message(String::from("\n") + data.as_str(), o, l, dstip);
            },
            _ => {
                o.lock().unwrap().
                    println(String::from("Could not read file."), color::WHITE);
            }
        }
        return;
    }

    match txt.as_str() {
        "/help" => {
            help_message(o.clone());
        },
        "/uptime" | "/up" => {
            o.lock().unwrap().
                println(format!("up {}", decode_uptime(state.uptime())), color::WHITE);
        },
        _ => {
            o.lock().unwrap().
                println(String::from("Unknown command. Type /help to see a list of commands."), color::WHITE);
        }
    };
}

pub fn send_message(txt: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String) {

    let msg = Message::new(dstip, txt.clone().into_bytes());
    // TODO no lock here -> if sending wants to write a message it could dead lock
    let mut out = o.lock().unwrap();
    let fm = time::strftime("%R", &time::now()).unwrap();
    out.println(format!("{} [you] says: {}", fm, txt), color::WHITE);
    match l.send(msg) {
        Ok(_) => {
            let fm = time::strftime("%R", &time::now()).unwrap();
            out.println(format!("{} transmitting...", fm), color::BLUE);
        }
        Err(e) => { match e {
            Errors::MessageTooBig => { out.println(format!("Message too big."), color::RED); }
            Errors::SendFailed => { out.println(format!("Sending of message failed."), color::RED); }
            Errors::EncryptionError => {out.println(format!("Encryption failed."), color::RED); }
        }}
    }
}


fn decode_uptime(t: i64) -> String {

    let days = t / 86400;
    if days > 0 {
        if days > 1 {
            format!("{} days ({} seconds)", days, t)
        } else {
            format!("{} day ({} seconds)", days, t)
        }
    } else {
        format!("{} seconds", t)
    }
}
