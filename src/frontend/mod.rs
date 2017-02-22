extern crate term;

use self::term::color;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

mod humaninterface_ncurses;
mod humaninterface;
mod humaninterface_std;
mod callbacks;

use frontend::humaninterface::Output;
use frontend::callbacks::Callbacks;
use super::IncomingMessage;

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

pub struct Gui {
    pub o: Arc<Mutex<HiOut>>,
    pub i: HiIn,
    pub status_tx: Sender<String>
}

pub fn gui() -> Gui {
    let o = Arc::new(Mutex::new(HiOut::new()));
    Gui {
        o: o.clone(), // human interface for output
        i: HiIn::new(), // human interface for input
        status_tx: status_message_loop(o.clone())
    }
}

pub fn recv_loop(o: Arc<Mutex<HiOut>>, rx: Receiver<IncomingMessage>) {

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
                o.lock().unwrap()
                    .println(format!("recv_loop: failed to receive message. {:?}", e), color::RED);
            }
        }
    }});
}

pub fn status_message_loop(o: Arc<Mutex<HiOut>>) -> Sender<String> {

    let (tx, rx) = channel::<String>();

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                // TODO use s.th.  like debug, info, ...
                if msg.starts_with("") { // dummy to use variable

                }
                /*
                o.lock().unwrap()
                    .println(msg, color::YELLOW);
                */
            }
            Err(e) => {
                o.lock().unwrap()
                    .println(format!("status_message_loop: failed. {:?}", e), color::RED);
            }
        }
    }});

    tx
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
