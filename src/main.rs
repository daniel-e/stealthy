mod logo;
mod humaninterface;
mod humaninterface_std;
mod humaninterface_ncurses;
mod callbacks;
mod tools;
mod rsatools;

extern crate getopts;
extern crate term;
extern crate stealthy;
extern crate time;

extern crate crypto as cr;

use std::{env, thread};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use getopts::Options;
use term::color;
use std::fs::File;
use std::io::{BufRead, BufReader};

use cr::sha1::Sha1;
use cr::digest::Digest;

use stealthy::{Message, IncomingMessage, Errors, Layers};
use humaninterface::{Input, Output, UserInput, ControlType};
use callbacks::Callbacks;
use tools::{read_file, insert_delimiter, read_bin_file};
//use rsatools::key_as_der;

#[cfg(not(feature="usencurses"))]
use humaninterface_std::{StdIn, StdOut};
#[cfg(not(feature="usencurses"))]
type HiIn = StdIn;
#[cfg(not(feature="usencurses"))]
type HiOut = StdOut;

#[cfg(feature="usencurses")]
use humaninterface_ncurses::{NcursesIn, NcursesOut};
#[cfg(feature="usencurses")]
type HiIn = NcursesIn;
#[cfg(feature="usencurses")]
type HiOut = NcursesOut;

fn status_message_loop(o: Arc<Mutex<HiOut>>) -> Sender<String> {

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

fn recv_loop(o: Arc<Mutex<HiOut>>, rx: Receiver<IncomingMessage>) {

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                let mut out = o.lock().unwrap();
                match msg {
                    IncomingMessage::New(msg) =>    { out.new_msg(msg); }
                    IncomingMessage::Ack(id)  =>    { out.ack_msg(id); }
                    IncomingMessage::Error(_, s)     => { out.err_msg(s); }
                    IncomingMessage::FileUpload(msg) => {
                        let fname = msg.get_filename();
                        if fname.is_some() {
                            let f = fname.unwrap();
                            let p = f.iter().position(|x| *x == 0 as u8);
                            match p {
                                Some)
                            }
                            out.new_file(msg, fname.unwrap());
                        }
                    }
                }
            }
            Err(e) => {
                o.lock().unwrap()
                    .println(format!("recv_loop: failed to receive message. {:?}", e), color::RED);
            }
        }
    }});
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


fn help_message(o: Arc<Mutex<HiOut>>) {

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

#[derive(Clone, Debug)]
pub struct GlobalState {
    start_time: time::Timespec
}

static mut GLOBAL_STATE: Option<GlobalState> = None;

// returns the uptime of stealthy in seconds
fn uptime() -> i64 {
    // TODO access to global state needs to be synchronized
    unsafe {
        time::get_time().sec - GLOBAL_STATE.clone().unwrap().start_time.sec
    }
}

fn init_global_state() {
    unsafe {
        GLOBAL_STATE = Some(GlobalState {
            start_time: time::get_time(),
        })
    };
}

fn parse_command(txt: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String) {
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

    if txt.starts_with("/upload ") {
        let (_, b) = txt.as_str().split_at(8);
        match read_bin_file(b) {
            Ok(data) => {
                send_file(data, b.to_string(), o, l, dstip);
            },
            Err(s) => {
                o.lock().unwrap().println(String::from(s), color::WHITE);
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
                println(format!("up {}", decode_uptime(uptime())), color::WHITE);
        },
        _ => {
            o.lock().unwrap().
                println(String::from("Unknown command. Type /help to see a list of commands."), color::WHITE);
        }
    };
}

fn send_file(data: Vec<u8>, fname: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String) {

    let n = data.len();
    let msg = Message::file_upload(dstip, fname.clone(), data);

    // TODO no lock here -> if sending wants to write a message it could dead lock
    let mut out = o.lock().unwrap();
    let fm = time::strftime("%R", &time::now()).unwrap();
    out.println(format!("{} [you] sending file '{}' with {} bytes...", fm, fname, n), color::YELLOW);

    // send message
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

fn send_message(txt: String, o: Arc<Mutex<HiOut>>, l: &Layers, dstip: String) {

    let msg = Message::new(dstip, txt.clone().into_bytes());
    // TODO no lock here -> if sending wants to write a message it could dead lock
    let mut out = o.lock().unwrap();
    let fm = time::strftime("%R", &time::now()).unwrap();
    out.println(format!("{} [you] says: {}", fm, txt), color::WHITE);

    // send message
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

fn input_loop(o: Arc<Mutex<HiOut>>, i: HiIn, l: Layers, dstip: String) {

    // read from human interface until user enters control-d and send the
    // message via the network layer
    loop { match i.read_line() {
            Some(ui) => {
                match ui {
                    UserInput::Line(s) => {
                        let txt = s.trim_right().to_string();
                        if txt.len() > 0 {
                            if txt.starts_with("/") {
                                parse_command(txt, o.clone(), &l, dstip.clone());
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
    init_global_state();

    // parse command line arguments
	let r = parse_arguments();
    let args = if r.is_some() { r.unwrap() } else { return };

    let o = Arc::new(Mutex::new(HiOut::new()));    // human interface for output
    let i = HiIn::new();                           // human interface for input
    let status_tx = status_message_loop(o.clone());

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

    input_loop(o.clone(), i, layer.layers, args.dstip);
}

struct Arguments {
    pub device: String,
    pub dstip: String,
    pub hybrid_mode: bool,
    pub secret_key: String,
    pub rcpt_pubkey_file: String,
    pub privkey_file: String,
    pub pubkey_file: String,
}

fn get_key_from_home() -> Option<String> {
    match env::home_dir() {
        Some(mut path) => {
            path.push(".stealthy/key");
            match File::open(path) {
                Ok(f) => {
                    let mut reader = BufReader::new(f);
                    let mut key = String::new();
                    match reader.read_line(&mut key) {
                        Ok(_) => Some(key.trim().to_string()),
                        _ => None
                    }
                }
                _ => None
            }
        },
        None => None
    }
}

fn parse_arguments() -> Option<Arguments> {

    static DEFAULT_SECRET_KEY: &'static str = "11111111111111111111111111111111";

	// parse comand line options
	let args : Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("i", "dev", "set the device where to listen for messages", "device");
	opts.optopt("d", "dst", "set the IP where messages are sent to", "IP");
	opts.optopt("e", "enc", "set the encryption key", "key");
	opts.optopt("r", "recipient", "recipient's public key in PEM format used for encryption", "filename");
	opts.optopt("p", "priv", "your private key in PEM format used for decryption", "filename");
    opts.optopt("q", "pub", "your public key in PEM format", "filename");
	opts.optflag("h", "help", "print this message");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!(f.to_string()) }
	};

    let hybrid_mode = matches.opt_present("r") || matches.opt_present("p");

	if matches.opt_present("h") ||
            (hybrid_mode && !(matches.opt_present("r") && matches.opt_present("p") && matches.opt_present("q"))) {

		let brief = format!("Usage: {} [options]", args[0]);
		println!("{}", opts.usage(&brief));
		return None;
	}

    // 1) If option -e is given use this key.
    // 2) If key exists in home directory use this key.
    // 3) Use default key.
    let key = matches.opt_str("e")
        .unwrap_or(get_key_from_home().unwrap_or(DEFAULT_SECRET_KEY.to_string()));

    Some(Arguments {
        device:       matches.opt_str("i").unwrap_or("lo".to_string()),
        dstip:        matches.opt_str("d").unwrap_or("127.0.0.1".to_string()),
        secret_key:   key,
        hybrid_mode:  hybrid_mode,
        rcpt_pubkey_file:  matches.opt_str("r").unwrap_or("".to_string()),
        privkey_file: matches.opt_str("p").unwrap_or("".to_string()),
        pubkey_file:  matches.opt_str("q").unwrap_or("".to_string()),
    })
}
