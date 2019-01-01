mod logo;
mod tools;
mod rsatools;
mod arguments;
mod console;
mod ui_termion;

extern crate crypto as cr;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use rand::{thread_rng, Rng};

use cr::sha1::Sha1;
use cr::digest::Digest;

use stealthy::{Message, IncomingMessage, Layers, Layer};
use crate::tools::{read_file, insert_delimiter, read_bin_file, write_data, decode_uptime, without_dirs};
use crate::arguments::{parse_arguments, Arguments};
use crate::console::ConsoleMessage;

use crate::ui_termion::{UserInput, ControlType, TermIn, TermOut, ItemType};
use crate::ui_termion::Model;
use crate::ui_termion::Item;
use crate::ui_termion::Symbol;

type HInput = TermIn;
type HOutput = TermOut;


// Receives messages via channel and writes the message to the screen.
fn status_message_loop(o: Sender<ConsoleMessage>) -> Sender<String> {

    let (tx, rx) = channel::<String>();

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(_msg) => {
                console::status(o.clone(), _msg);
                // TODO use s.th.  like debug, info, ...
                //if msg.starts_with("") { // dummy to use variable

                //}
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
        "/uptime, /up        - uptime",
        "/cat <filename>     - send content of an UTF-8 encoded text file",
        "/upload <filename>  - send binary file",
        "Keys:",
        "arrow up            - scroll to older messages",
        "arrow down          - scroll to latest messages",
        "page up             - scroll one page up",
        "page down           - scroll one page down",
        "end                 - scroll to last message in buffer",
        "pos1                - scroll to first message in buffer",
        "esc or ctrl+d       - quit"
    ];

    for v in lines {
        console::msg(o.clone(), String::from(v), ItemType::Info)
    }
    console::raw(o, String::from(" "), ItemType::Info);
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
                console::msg(o.clone(), String::from("Transmitting data ..."), ItemType::Info);
                send_message(String::from("\n") + data.as_str(), o.clone(), l, dstip);
            },
            _ => {
                console::msg(o.clone(), String::from("Could not read file."), ItemType::Error);
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
                console::msg(o, String::from(s), ItemType::Error);
            }
        }
        return;
    }

    match txt.as_str() {
        "/help" => {
            help_message(o.clone());
        },
        "/uptime" | "/up" => {
            console::msg(o, format!("up {}", decode_uptime(uptime())), ItemType::Info);
        },
        _ => {
            console::msg(o, String::from("Unknown command. Type /help to see a list of commands."), ItemType::Info);
        }
    };
}

fn do_send(msg: Message, l: &Layers, id: u64, background: bool) {
    l.send(msg, id, background);
}

fn send_file(data: Vec<u8>, fname: String, o: Sender<ConsoleMessage>, l: &Layers, dstip: String) {

    let n = data.len();
    let msg = Message::file_upload(dstip, without_dirs(&fname), data);

    let id = rand::random::<u64>();
    console::msg_item(
        o.clone(),
        Item::new(
            format!("[you] sending file '{}' with {} bytes...", fname, n),
            ItemType::MyMessage
        ).symbol(Symbol::Transmitting).id(id)
    );
    do_send(msg, l, id, true);
}

fn send_message(txt: String, o: Sender<ConsoleMessage>, l: &Layers, dstip: String) {

    let msg = Message::new(dstip, txt.clone().into_bytes());

    let id = rand::random::<u64>();
    console::msg_item(
        o.clone(),
        Item::new(
            format!("[you] {}", txt),
            ItemType::MyMessage
        ).symbol(Symbol::Transmitting).id(id)
    );
    do_send(msg, l, id, false);
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
        console::raw(o.clone(), l, ItemType::Introduction);
    }

    let v = vec![
        format!("The most secure ICMP messenger."),
        format!(" "),
        format!("┌─────────────────────┬──────────────────┐"),
        format!("│ Listening on device │ {}               │", args.device),
        format!("│ Talking to IP       │ {:16} │", args.dstip),
        format!("└─────────────────────┴──────────────────┘"),
        format!(" "),
        format!("Type /help to get a list of available commands."),
        format!("Esc or Ctrl+D to quit.")
    ];
    for i in v {
        console::raw(o.clone(), i, ItemType::Introduction);
    }

    if args.hybrid_mode {
        let mut h = Sha1::new();

        h.input(&layer.layers.encryption_key());
        let s = insert_delimiter(&h.result_str());
        console::raw(o.clone(), format!("Hash of encryption key : {}", s), ItemType::Introduction);

        h.reset();
        h.input(&rsatools::key_as_der(&read_file(&args.pubkey_file).unwrap()));
        let q = insert_delimiter(&h.result_str());
        console::raw(o.clone(), format!("Hash of your public key: {}", q), ItemType::Introduction);
    }
    console::raw(o.clone(), format!(" "), ItemType::Introduction);
    console::raw(o.clone(), format!("Happy chatting..."), ItemType::Introduction);
    console::raw(o.clone(), format!(" "), ItemType::Introduction);
}




fn input_loop(o: Sender<ConsoleMessage>, mut i: HInput, l: Layers, dstip: String) {

    // read from human interface until user enters control-d and send the
    // message via the network layer
    loop { match i.read_char() {
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
                },
                UserInput::Refresh => {
                    send_channel(o.clone(), ConsoleMessage::Refresh);
                }
            }
        }
        _ => { break; }
    }}
    send_channel(o, ConsoleMessage::Exit);

    // Sleep some time so that the output has some time to reset the terminal.
    thread::sleep(Duration::from_millis(100));
}

fn init_screen(model: Arc<Mutex<Model>>) -> Sender<ConsoleMessage> {
    let (tx, rx) = channel::<ConsoleMessage>();
    let mut o = HOutput::new(model);

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                match msg {
                    ConsoleMessage::TextMessage(item) => {
                        o.println(item);
                    },
                    ConsoleMessage::Ack(id) => {
                        o.ack(id);
                    }
                    ConsoleMessage::Exit => {
                        o.close();
                        break;
                    },
                    ConsoleMessage::ScrollUp => {
                        o.scroll_up();
                    },
                    ConsoleMessage::ScrollDown => {
                        o.scroll_down();
                    },
                    ConsoleMessage::Refresh => {
                        o.refresh();
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

    let model = Arc::new(Mutex::new(Model::new()));

    let output = init_screen(model.clone());

    // Input
    let i = HInput::new(model);

    // Creates a thread which waits for messages on a channel to be written to o.
    let status_tx = status_message_loop(output.clone());

    let layer = get_layer(&args, status_tx);

    welcome(&args, output.clone(), &layer);

    // this is the loop which handles messages received via rx
    recv_loop(output.clone(), layer.rx);

    input_loop(output.clone(), i, layer.layers, args.dstip);
}

