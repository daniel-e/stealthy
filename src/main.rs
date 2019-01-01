mod logo;
mod tools;
mod rsatools;
mod arguments;
mod console;
mod ui_termion;
mod model;
mod ui_in;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use rand::{thread_rng, Rng};

use crypto::sha1::Sha1;
use crypto::digest::Digest;

use stealthy::{Message, IncomingMessage, Layers, Layer};
use crate::tools::{read_file, insert_delimiter, read_bin_file, write_data, decode_uptime, without_dirs};
use crate::arguments::{parse_arguments, Arguments};
use crate::console::ConsoleMessage;

use crate::ui_termion::TermOut;
use crate::ui_in::{TermIn, UserInput};
use crate::model::{ItemType, Model, Item, Symbol};

type HInput = TermIn;
type HOutput = TermOut;
type ArcModel = Arc<Mutex<Model>>;
type ArcOut = Arc<Mutex<HOutput>>;
type Channel = Sender<ConsoleMessage>;

fn help_message(o: Channel) {

    write_lines(o, &vec![
        "Commands always start with a slash:",
        " ",
        "/help               - this help message",
        "/uptime, /up        - uptime",
        "/cat <filename>     - send content of an UTF-8 encoded text file",
        "/upload <filename>  - send binary file",
        " ",
        "Keys:",
        " ",
        "arrow up            - scroll to older messages",
        "arrow down          - scroll to latest messages",
        "page up             - scroll one page up",
        "page down           - scroll one page down",
        "end                 - scroll to last message in buffer",
        "esc or ctrl+d       - quit",
        " "
    ], ItemType::Info);
}

fn write_lines(o: Channel, lines: &[&str], typ: ItemType) {

    for v in lines {
        console::raw(o.clone(), String::from(*v), typ.clone())
    }
}

fn recv_loop(o: Channel, rx: Receiver<IncomingMessage>) {

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => process_incoming_message(o.clone(), msg),
            Err(e) => console::error(o.clone(), format!("recv_loop: failed to receive message. {:?}", e))
        }}
    });
}

fn process_incoming_message(o: Channel, msg: IncomingMessage) {

    match msg {
        IncomingMessage::New(msg) => { console::new_msg(o.clone(), msg); }
        IncomingMessage::Ack(id) => { console::ack_msg(o.clone(), id); }
        IncomingMessage::Error(_, s) => { console::error(o.clone(), s); }
        IncomingMessage::FileUpload(msg) => { process_upload(o.clone(), msg) }
    }
}

fn random_str(n: usize) -> String {

    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = thread_rng();
    String::from_utf8(
        (0..n).map(|_| { chars[rng.gen::<usize>() % chars.len()] }).collect()
    ).unwrap()
}

fn process_upload(o: Channel, msg: Message) {

    if msg.get_filename().is_none() {
        console::error(o.clone(), format!("Could not get filename of received file upload."));
        return;
    } else if msg.get_filedata().is_none() {
        console::error(o.clone(), format!("Could not get data of received file upload."));
        return;
    }

    let fname = msg.get_filename().unwrap();
    let data = msg.get_filedata().unwrap();
    let dst = format!("/tmp/stealthy_{}_{}", random_str(10), &fname);
    console::new_file(o.clone(), msg, fname);

    if write_data(&dst, data) {
        console::status(o.clone(), format!("File written to '{}'.", dst));
    } else {
        console::error(o.clone(), format!("Could not write data of received file upload."));
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
asdf
}

fn init_global_state() {
    unsafe {
        GLOBAL_STATE = Some(GlobalState {
            start_time: time::get_time(),
        })
    };
}

fn parse_command(txt: String, o: Channel, l: &Layers, dstip: String) {
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

fn msg_transmitting(o: Channel, id: u64, s: String) {

    console::msg_item(
        o,Item::new(s,ItemType::MyMessage).symbol(Symbol::Transmitting).id(id)
    );
}

fn send_file(data: Vec<u8>, fname: String, o: Channel, l: &Layers, dstip: String) {

    let n = data.len();
    let msg = Message::file_upload(dstip, without_dirs(&fname), data);

    let id = rand::random::<u64>();
    msg_transmitting(o, id, format!("[you] sending file '{}' with {} bytes...", fname, n));
    l.send(msg, id, true);
}

fn send_message(txt: String, o: Channel, l: &Layers, dstip: String) {

    let msg = Message::new(dstip, txt.clone().into_bytes());

    let id = rand::random::<u64>();
    msg_transmitting(o, id, format!("[you] {}", txt));
    l.send(msg, id, false);
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

fn welcome(args: &Arguments, o: Channel, layer: &Layer) {
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

    write_lines(
        o.clone(),
        v.iter().map(|x| x.as_str()).collect::<Vec<_>>().as_slice(),
        ItemType::Introduction
    );

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


fn status_message_loop(o: Channel) -> Sender<String> {

    let (tx, rx) = channel::<String>();
    thread::spawn(move || { loop { match rx.recv() {
        Ok(msg) => console::status(o.clone(), msg),
        Err(er) => console::error(o.clone(), format!("status_message_loop: failed. {:?}", er))
    }}});
    tx
}

fn input_loop(o: Channel, mut i: HInput, l: Layers, dstip: String, model: ArcModel, out: ArcOut) {

    loop { match i.read_char() {
        UserInput::Character(buf) => {
            model.lock().unwrap().update_input(buf);
            out.lock().unwrap().refresh();
        },
        UserInput::Escape | UserInput::CtrlD => {
            out.lock().unwrap().close();
            o.send(ConsoleMessage::Exit).expect("Send failed.");
            break;
        },
        UserInput::ArrowDown => {
            out.lock().unwrap().scroll_down();
        },
        UserInput::ArrowUp => {
            out.lock().unwrap().scroll_up();
        },
        UserInput::Backspace => {
            model.lock().unwrap().apply_backspace();
            out.lock().unwrap().refresh();
        },
        UserInput::End => {
            out.lock().unwrap().key_end();
        },
        UserInput::PageDown => {
            out.lock().unwrap().page_down();
        },
        UserInput::PageUp => {
            out.lock().unwrap().page_up();
        },
        UserInput::Enter => {
            let s = model.lock().unwrap().apply_enter();
            out.lock().unwrap().refresh();
            if s.len() > 0 {
                if s.starts_with("/") {
                    parse_command(s, o.clone(), &l, dstip.clone());
                } else {
                    send_message(s, o.clone(), &l, dstip.clone());
                }
            }
        }
    }}
}

fn init_screen(model: ArcModel, out: ArcOut) -> Channel {

    // The sender "tx" is used at other locations to send messages to the output.
    let (tx, rx) = channel::<ConsoleMessage>();

    thread::spawn(move || {
        loop { match rx.recv().unwrap() {
            ConsoleMessage::TextMessage(item) => {
                model.lock().unwrap().add_message(item.clone());
                out.lock().unwrap().adjust_scroll_offset(item);
            },
            ConsoleMessage::Ack(id) => {
                model.lock().unwrap().ack(id);
                out.lock().unwrap().refresh();
            },
            // We need this as otherwise "out" is not dropped and the terminal state
            // is not restored.
            ConsoleMessage::Exit => {
                break;
            }
        }}
    });
    tx
}

fn main() {
    init_global_state();

    // Parse command line arguments.
	let args = parse_arguments().expect("Cannot parse arguments");;

    // The model stores all information which is required to show the screen.
    let model = Arc::new(Mutex::new(Model::new()));

    let out = Arc::new(Mutex::new(HOutput::new(model.clone())));

    let tx = init_screen(model.clone(), out.clone());

    // Used to receive user input.
    let i = HInput::new();

    // Creates a thread which waits for messages on a channel to be written to out.
    let status_tx = status_message_loop(tx.clone());

    let layer = get_layer(&args, status_tx);

    welcome(&args, tx.clone(), &layer);

    // this is the loop which handles messages received via rx
    recv_loop(tx.clone(), layer.rx);

    input_loop(tx.clone(), i, layer.layers, args.dstip, model, out);
}

