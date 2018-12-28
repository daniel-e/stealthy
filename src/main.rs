mod logo;
mod humaninterface;
mod humaninterface_ncurses;
mod callbacks;
mod tools;
mod rsatools;
mod arguments;
mod console;

extern crate getopts;
extern crate term;
extern crate stealthy;
extern crate time;
extern crate rand;
extern crate dirs;

extern crate crypto as cr;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use term::color;
use rand::{thread_rng, Rng};

use cr::sha1::Sha1;
use cr::digest::Digest;

use stealthy::{Message, IncomingMessage, Errors, Layers};
use humaninterface::{Input, Output, UserInput, ControlType};
use tools::{read_file, insert_delimiter, read_bin_file, write_data, decode_uptime, without_dirs};
use arguments::{parse_arguments, Arguments};
use stealthy::Layer;
use console::ConsoleMessage;

use humaninterface_ncurses::{NcursesIn, NcursesOut};
use std::time::Duration;

type HInput = NcursesIn;
type HOutput = NcursesOut;


// Receives messages via channel and writes the message to the screen.
fn status_message_loop(o: Sender<ConsoleMessage>) -> Sender<String> {

    let (tx, rx) = channel::<String>();

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                // TODO use s.th.  like debug, info, ...
                if msg.starts_with("") { // dummy to use variable

                }
            }
            Err(e) => {
                console::error(o.clone(), format!("status_message_loop: failed. {:?}", e));
            }
        }
    }});

    tx
}

fn recv_loop(o: Sender<ConsoleMessage>, rx: Receiver<IncomingMessage>) {

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                match msg {
                    IncomingMessage::New(msg)        => { console::new_msg(o.clone(), msg); }
                    IncomingMessage::Ack(id)         => { console::ack_msg(o.clone(), id); }
                    IncomingMessage::Error(_, s)     => { console::error(o.clone(), s); }
                    IncomingMessage::FileUpload(msg) => {
                        match msg.get_filename() {
                            Some(fname) => {
                                let fdata = msg.get_filedata();
                                let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
                                let mut rng = thread_rng();
                                let b: Vec<u8> = (0..10).map(|_| {chars[rng.gen::<usize>() % chars.len()]}).collect();
                                let r = String::from_utf8(b).expect("Invalid characters.");
                                let dst = format!("/tmp/stealthy_{}_{}", r, &fname);
                                console::new_file(o.clone(), msg, fname);
                                match fdata {
                                    Some(data) => {
                                        if write_data(&dst, data) {
                                            console::status(o.clone(), format!("File written to '{}'.", dst));
                                        } else {
                                            console::error(o.clone(), format!("Could not write data of received file upload."));
                                        }
                                    },
                                    _ => { console::error(o.clone(), format!("Could not get data of received file upload.")); }
                                }
                            },
                            _ => { console::error(o.clone(), format!("Could not get filename of received file upload.")); }
                        }
                    }
                }
            }
            Err(e) => {
                console::error(o.clone(), format!("recv_loop: failed to receive message. {:?}", e));
            }
        }
    }});
}

fn help_message(o: Sender<ConsoleMessage>) {

    let lines = vec![
        "Commands always start with a slash:",
        "/help               - this help message",
        "arrow up            - scroll to older messages",
        "arrow down          - scroll to latest messages",
        "/uptime, /up        - uptime",
        "/cat <filename>     - send content of an UTF-8 encoded text file",
        "/upload <filename>  - send binary file"
    ];

    for v in lines {
        console::msg(o.clone(), String::from(v), color::WHITE)
    }
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

fn parse_command(txt: String, o: Sender<ConsoleMessage>, l: &Layers, dstip: String) {
    // TODO: find more elegant solution for this
    if txt.starts_with("/cat ") {
        // TODO split_at works on bytes not characters
        let (_, b) = txt.as_str().split_at(5);
        match read_file(b) {
            Ok(data) => {
                console::msg(o.clone(), String::from("Transmitting data ..."), color::WHITE);
                send_message(String::from("\n") + data.as_str(), o.clone(), l, dstip);
            },
            _ => {
                console::msg(o.clone(), String::from("Could not read file."), color::WHITE);
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
                console::msg(o, String::from(s), color::WHITE);
            }
        }
        return;
    }

    match txt.as_str() {
        "/help" => {
            help_message(o.clone());
        },
        "/uptime" | "/up" => {
            console::msg(o, format!("up {}", decode_uptime(uptime())), color::WHITE);
        },
        _ => {
            console::msg(o, String::from("Unknown command. Type /help to see a list of commands."), color::WHITE);
        }
    };
}

fn send_file(data: Vec<u8>, fname: String, o: Sender<ConsoleMessage>, l: &Layers, dstip: String) {

    let n = data.len();
    let msg = Message::file_upload(dstip, without_dirs(&fname), data);

    // TODO no lock here -> if sending wants to write a message it could dead lock
    let fm = time::strftime("%R", &time::now()).unwrap();
    console::msg(o.clone(), format!("{} [you] sending file '{}' with {} bytes...", fm, fname, n), color::YELLOW);

    // send message
    match l.send(msg) {
        Ok(_) => {
            let fm = time::strftime("%R", &time::now()).expect("send_file: strftime failed");
            console::msg(o, format!("{} transmitting...", fm), color::BLUE);
        }
        Err(e) => { match e {
            Errors::MessageTooBig => { console::msg(o, format!("Message too big."), color::RED); }
            Errors::SendFailed => { console::msg(o, format!("Sending of message failed."), color::RED); }
            Errors::EncryptionError => { console::msg(o, format!("Encryption failed."), color::RED); }
        }}
    }
}

fn send_message(txt: String, o: Sender<ConsoleMessage>, l: &Layers, dstip: String) {

    let msg = Message::new(dstip, txt.clone().into_bytes());
    // TODO no lock here -> if sending wants to write a message it could dead lock
    let fm = time::strftime("%R", &time::now()).unwrap();
    console::msg(o.clone(), format!("{} [you] says: {}", fm, txt), color::WHITE);

    // send message
    match l.send(msg) {
        Ok(_) => {
            let fm = time::strftime("%R", &time::now()).unwrap();
            console::msg(o, format!("{} transmitting...", fm), color::BLUE);
        }
        Err(e) => { match e {
            Errors::MessageTooBig => { console::msg(o, format!("Message too big."), color::RED); }
            Errors::SendFailed => { console::msg(o, format!("Sending of message failed."), color::RED); }
            Errors::EncryptionError => { console::msg(o, format!("Encryption failed."), color::RED); }
        }}
    }
}

fn send_channel(o: Sender<ConsoleMessage>, c: ConsoleMessage) {
    o.send(c).expect("Could not send message.");
}

fn get_layer(args: &Arguments, status_tx: Sender<String>) -> Layer {
    let ret =
        if args.hybrid_mode {
            // use asymmetric encryption
            Layers::asymmetric(&args.rcpt_pubkey_file, &args.privkey_file, &args.device, status_tx)
        } else {
            // use symmetric encryption
            Layers::symmetric(&args.secret_key, &args.device, status_tx)
        };
    ret.expect("Initialization failed.")
}

fn welcome(args: &Arguments, o: Sender<ConsoleMessage>, layer: &Layer) {
    for l in logo::get_logo() {
        console::msg(o.clone(), l, color::GREEN);
    }
    console::msg(o.clone(), format!("Welcome to stealthy! The most secure ICMP messenger."), color::YELLOW);
    console::msg(o.clone(), format!("Type /help to get a list of available commands."), color::YELLOW);
    console::msg(o.clone(), format!(""), color::WHITE);
    console::msg(o.clone(), format!("device is {}, destination ip is {}", args.device, args.dstip), color::WHITE);
    if args.hybrid_mode {
        let mut h = Sha1::new();

        h.input(&layer.layers.encryption_key());
        let s = insert_delimiter(&h.result_str());
        console::msg(o.clone(), format!("Hash of encryption key : {}", s), color::YELLOW);

        h.reset();
        h.input(&rsatools::key_as_der(&read_file(&args.pubkey_file).unwrap()));
        let q = insert_delimiter(&h.result_str());
        console::msg(o.clone(), format!("Hash of your public key: {}", q), color::YELLOW);
    }
    console::msg(o.clone(), format!("Happy chatting...\n"), color::WHITE);
}




fn input_loop(o: Sender<ConsoleMessage>, i: HInput, l: Layers, dstip: String) {

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
                    match what {
                        ControlType::ArrowUp => { send_channel(o.clone(), ConsoleMessage::ScrollUp); },
                        ControlType::ArrowDown => { send_channel(o.clone(), ConsoleMessage::ScrollDown); }
                    }
                }
            }
        }
        _ => { break; }
    }}
    send_channel(o, ConsoleMessage::Exit);
}

fn init_screen() -> Sender<ConsoleMessage> {
    let (tx, rx) = channel::<ConsoleMessage>();
    let mut o = HOutput::new();

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                match msg {
                    ConsoleMessage::TextMessage(msg) => {
                        o.println(msg.msg, msg.col);
                    },
                    ConsoleMessage::Exit => {
                        o.close();
                        break;
                    },
                    ConsoleMessage::ScrollUp => {
                        o.scroll_up();
                    },
                    ConsoleMessage::ScrollDown => {
                        o.scroll_down();
                    }
                }
            }
            Err(_e) => {
                o.close();
                break;
            }
        }}}
    );

    tx
}



fn main() {
    init_global_state();

    // parse command line arguments
	let args = parse_arguments().expect("Cannot parse arguments");;

    let output = init_screen();
    // Output
    //let o = Arc::new(Mutex::new(HOutput::new()));

    // Input
    let i = HInput::new();

    // Creates a thread which waits for messages on a channel to be written to o.
    let status_tx = status_message_loop(output.clone());

    let layer = get_layer(&args, status_tx);

    welcome(&args, output.clone(), &layer);


    // this is the loop which handles messages received via rx
    recv_loop(output.clone(), layer.rx);

    thread::sleep(Duration::from_millis(100));

    input_loop(output.clone(), i, layer.layers, args.dstip);
}

