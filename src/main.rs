mod outputs;
mod tools;
mod rsatools;
mod arguments;
mod console;
mod view;
mod model;
mod keyboad;
mod message;
mod layer;
mod cryp;
mod delivery;
mod binding;
mod iptools;
mod blowfish;
mod packet;
mod rsa;
mod error;
mod commands;

use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::message::{Message, IncomingMessage};
use crate::layer::{Layers, Layer};
use crate::tools::write_data;
use crate::iptools::IpAddresses;
use crate::arguments::{parse_arguments, Arguments};
use crate::console::ConsoleMessage;
use crate::view::View;
use crate::keyboad::{InputKeyboard, UserInput};
use crate::model::{ItemType, Model, Item};
use crate::model::Source;

type ArcModel = Arc<Mutex<Model>>;
type ArcView = Arc<Mutex<View>>;
pub type ConsoleSender = Sender<ConsoleMessage>;

/// Listens for incoming messages from the network.
fn recv_loop(o: ConsoleSender, rx: Receiver<IncomingMessage>) {

    thread::spawn(move || {
        loop { match rx.recv() {
            Ok(msg) => {
                match msg {
                    IncomingMessage::New(msg) => {
                        console::new_msg(o.clone(), msg);
                    }
                    IncomingMessage::Ack(id) => {
                        console::ack_msg(o.clone(), id);
                    }
                    IncomingMessage::Error(_, s) => {
                        console::error(o.clone(), s);
                    }
                    IncomingMessage::FileUpload(msg) => {
                        process_upload(o.clone(), msg)
                    }
                    IncomingMessage::AckProgress(id, done, total) => {
                        console::ack_msg_progress(o.clone(), id, done, total);
                    }
                }
            },
            Err(e) =>  {
                console::error(o.clone(), format!("recv_loop: failed to receive message. {:?}", e))
            }
        }}
    });
}

fn process_upload(o: ConsoleSender, msg: Message) {

    if msg.get_filename().is_none() {
        console::error(o.clone(), format!("Could not get filename of received file upload."));
        return;
    } else if msg.get_filedata().is_none() {
        console::error(o.clone(), format!("Could not get data of received file upload."));
        return;
    }

    let fname = msg.get_filename().unwrap();
    let data = msg.get_filedata().unwrap();
    let dst = format!("/tmp/stealthy_{}_{}", tools::random_str(10), &fname);
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
}

fn init_global_state() {
    unsafe {
        GLOBAL_STATE = Some(GlobalState {
            start_time: time::get_time(),
        })
    };
}


fn create_data(dstip: String, txt: &String) -> (Message, u64) {
    (Message::new(dstip, txt.clone().into_bytes()), rand::random::<u64>())
}

fn send_message(txt: String, o: ConsoleSender, l: &Layers, dstips: &IpAddresses) {

    let mut item = Item::new(format!("{}", txt), ItemType::MyMessage, model::Source::You);

    let v = dstips.as_strings()
        .iter()
        .map(|dstip| create_data(dstip.clone(), &txt))
        .collect::<Vec<_>>();

    for (_, id) in &v {
        item = item.add_id(*id);
    }
    console::msg_item(o.clone(),item);

    for (msg, id) in v {
        l.send(msg, id, false);
    }
}

fn get_layer(args: &Arguments, status_tx: Sender<String>, dstips: &IpAddresses) -> Layer {
    let ret =
        if args.hybrid_mode {
            // use asymmetric encryption
            Layers::asymmetric(&args.rcpt_pubkey_file, &args.privkey_file, &args.device, status_tx, dstips)
        } else {
            // use symmetric encryption
            Layers::symmetric(&args.secret_key, &args.device, status_tx, dstips)
        };
    ret.expect("Initialization failed.")
}

fn status_message_loop(o: ConsoleSender) -> Sender<String> {

    let (tx, rx) = channel::<String>();
    thread::spawn(move || { loop { match rx.recv() {
        Ok(msg) => console::status(o.clone(), msg),
        Err(er) => console::error(o.clone(), format!("status_message_loop: failed. {:?}", er))
    }}});
    tx
}

fn keyboad_loop(o: ConsoleSender, l: Layers, dstips: IpAddresses, model: ArcModel, view: ArcView) {
    let mut input = InputKeyboard::new();

    loop {
        let i = input.read_char();
        model.lock().unwrap().update_last_keypress();
        match i {
            UserInput::Character(buf) => {
                let mut v = vec![];
                for c in buf {
                    let mut m = model.lock().unwrap();
                    if c == 13 {
                        let s = m.apply_enter();
                        send_message(s, o.clone(), &l, &dstips);
                    } else {
                        v.push(c);
                        if String::from_utf8(v.clone()).is_ok() {
                            m.update_input(v.clone());
                            v.clear();
                        }
                    }
                }
                view.lock().unwrap().refresh();
            },
            UserInput::Escape | UserInput::CtrlD => {
                view.lock().unwrap().close();
                o.send(ConsoleMessage::Exit).expect("Send failed.");
                // Wait some seconds to give the thread in create_console_sender a chance to
                // release its view so that the terminal is recovered correctly.
                thread::sleep(Duration::from_millis(100));
                break;
            },
            UserInput::ArrowDown => {
                view.lock().unwrap().scroll_down();
            },
            UserInput::ArrowUp => {
                view.lock().unwrap().scroll_up();
            },
            UserInput::Backspace => {
                model.lock().unwrap().apply_backspace();
                view.lock().unwrap().refresh();
            },
            UserInput::End => {
                view.lock().unwrap().key_end();
            },
            UserInput::PageDown => {
                view.lock().unwrap().page_down();
            },
            UserInput::PageUp => {
                view.lock().unwrap().page_up();
            },
            UserInput::CtrlR => {
                view.lock().unwrap().toggle_raw_view();
            },
            UserInput::CtrlS => {
                model.lock().unwrap().toggle_scramble();
                view.lock().unwrap().refresh();
            },
            UserInput::Enter => {
                let s = model.lock().unwrap().apply_enter();
                view.lock().unwrap().refresh();
                if s.len() > 0 {
                    if s.starts_with("/") {
                        commands::parse_command(s, o.clone(), &l, &dstips);
                    } else {
                        send_message(s, o.clone(), &l, &dstips);
                    }
                }
            }
        }
    }
}

fn create_console_sender(model: ArcModel, view: ArcView) -> ConsoleSender {

    // The sender "tx" is used at other locations to send messages to the output.
    let (tx, rx) = channel::<ConsoleMessage>();

    thread::spawn(move || {
        loop { match rx.recv().unwrap() {
            ConsoleMessage::TextMessage(item) => {
                model.lock().unwrap().add_message(item.clone());
                view.lock().unwrap().adjust_scroll_offset(item);
            },
            ConsoleMessage::Ack(id) => {
                model.lock().unwrap().ack(id);
                view.lock().unwrap().refresh();
            },
            ConsoleMessage::AckProgress(id, done, total) => {
                model.lock().unwrap().ack_progress(id, done, total);
                view.lock().unwrap().refresh();
            },
            // We need this as otherwise "out" is not dropped and the terminal state
            // is not restored.
            ConsoleMessage::Exit => {
                break;
            },
            ConsoleMessage::SetScrambleTimeout(n) => {
                model.lock().unwrap().scramble_timeout = n;
            },
            ConsoleMessage::ScrambleTick => {
                let mut redraw = false;
                {
                    let mut m = model.lock().unwrap();
                    if !m.is_scrambled() {
                        let last_keypress = m.last_keypress();
                        if last_keypress.elapsed().unwrap().as_secs() > m.scramble_timeout as u64 {
                            m.scramble(true);
                            redraw = true;
                        }
                    }
                }
                if redraw {
                    view.lock().unwrap().refresh();
                }
            }
        }}
    });
    tx
}

fn scramble_trigger(o: ConsoleSender) {
    thread::spawn(move || {
       loop {
           thread::sleep(Duration::from_secs(1));
           o.send(ConsoleMessage::ScrambleTick).unwrap();
       }
    });
}

fn main() {
    init_global_state();

    // Parse command line arguments.
	let args = parse_arguments().expect("Cannot parse arguments");;

    let dstips = IpAddresses::from_comma_list(&args.dstip);

    // The model stores all information which is required to show the screen.
    let model = Arc::new(Mutex::new(Model::new()));

    let view = Arc::new(Mutex::new(View::new(model.clone())));

    let tx = create_console_sender(model.clone(), view.clone());

    // TODO replace status_message_loop by tx?
    // TODO have only one loop? for keyboard events, status message events and other events

    let layer = get_layer(&args, status_message_loop(tx.clone()), &dstips);

    outputs::welcome(&args, tx.clone(), &layer, &dstips);

    scramble_trigger(tx.clone());

    // this is the loop which handles messages received via rx
    recv_loop(tx.clone(), layer.rx);

    // Waits for data from the keyboard.
    // If data is received the model and the view will be updated.
    keyboad_loop(tx.clone(), layer.layers, dstips, model, view);

    // IMPORTANT! If the are threads which are using a clone of the view, the view isn't destroyed
    // properly and the terminal state is not restored.
}

