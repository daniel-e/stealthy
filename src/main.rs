mod logo;
mod tools;
mod rsatools;
mod options;

extern crate getopts;
extern crate term;
extern crate stealthy;
extern crate time;

extern crate crypto as cr;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use cr::sha1::Sha1;
use cr::digest::Digest;

use stealthy::{Message, IncomingMessage, Errors, Layers};
use tools::{read_file, insert_delimiter};
use options::parse_arguments;
use term::color;

mod frontend;
use frontend::gui;

//use rsatools::key_as_der;



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

#[derive(Clone, Debug, Copy)]
pub struct GlobalState {
    start_time: time::Timespec
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            start_time: time::get_time()
        }
    }

    pub fn uptime(&self) -> i64 {
        time::get_time().sec - self.start_time.sec
    }
}

fn parse_command(txt: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String, state: &GlobalState) {
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

fn send_message(txt: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String) {

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

fn input_loop(o: Arc<Mutex<HiOut>>, i: HiIn, l: Layers, dstip: String, state: &GlobalState) {

    // read from human interface until user enters control-d and send the
    // message via the network layer
    loop { match i.read_line() {
            Some(ui) => {
                match ui {
                    UserInput::Line(s) => {
                        let txt = s.trim_right().to_string();
                        if txt.len() > 0 {
                            if txt.starts_with("/") {
                                parse_command(txt, o.clone(), &l, dstip.clone(), state);
                            } else {
                                send_message(txt, o.clone(), &l, dstip.clone());
                            }
                        }
                    }
                    UserInput::Control(what) => {
                        let mut out = o.lock().unwrap();
                        match what {
                            ControlType::ArrowUp => out.scroll_up(),
                            ControlType::ArrowDown => out.scroll_down()
                        }
                    }
                }
            }
            _ => { break; }
    }}
    o.lock().unwrap().close();
}


fn main() {
    let state = GlobalState::new();

    // parse command line arguments
	let r = parse_arguments();
    let args = if r.is_some() { r.unwrap() } else { return };

    let gui = frontend::gui();

    //let o = Arc::new(Mutex::new(HiOut::new()));    // human interface for output
    //let i = HiIn::new();                           // human interface for input
    //let status_tx = status_message_loop(o.clone());

    let status_tx = gui.status_tx.clone();

    let ret =
        if args.hybrid_mode {
            // use asymmetric encryption
            Layers::asymmetric(&args.rcpt_pubkey_file, &args.privkey_file, &args.device, status_tx)  // network layer
        } else {
            // use symmetric encryption
            Layers::symmetric(&args.secret_key, &args.device, status_tx)  // network layer
        };

    if ret.is_err() {
        // TODO is this message visible when in curses
        println!("Initialization failed.");
        return;
    }

    let layer = ret.unwrap();

    // this is the loop which handles messages received via rx
    recv_loop(o.clone(), layer.rx);

    {
        let mut out = o.lock().unwrap();
        out.println(logo::get_logo(), color::GREEN);
        out.println(format!("device is {}, destination ip is {}", args.device, args.dstip), color::WHITE);
        if args.hybrid_mode {
            let mut h = Sha1::new();

            h.input(&layer.layers.encryption_key());
            let s = insert_delimiter(&h.result_str());
            out.println(format!("Hash of encryption key : {}", s), color::YELLOW);

            h.reset();
            h.input(&rsatools::key_as_der(&read_file(&args.pubkey_file).unwrap()));
            let q = insert_delimiter(&h.result_str());
            out.println(format!("Hash of your public key: {}", q), color::YELLOW);
        }
        out.println(format!("You can now start writing ...\n"), color::WHITE);
    }

    input_loop(o.clone(), i, layer.layers, args.dstip, &state);
}
