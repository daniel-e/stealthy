mod logo;
mod tools;
mod options;
mod frontend;
mod crypt;
mod delivery;
mod misc;
mod binding;
mod packet;

use std::sync::mpsc::Receiver;
use std::thread;
use misc::{Layers, IncomingMessage};
use tools::read_file;
use options::parse_arguments;
use frontend::{Gui, WHITE, GREEN, YELLOW};
use crypt::{hash_of, rsatools};

pub fn recv_loop(gui: &Gui, rx: Receiver<IncomingMessage>) {
    let tx = gui.get_channel();
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(msg) => tx.send(msg).unwrap(),
                Err(e)  => { panic!("failed to receive message: {:?}", e); }
            }
        }
    });
}

fn main() {
    // parse command line arguments
	let r = parse_arguments();
    let args = if r.is_some() { r.unwrap() } else { return };

    let gui = Gui::new();

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
    recv_loop(&gui, layer.rx);

    gui.println(logo::get_logo(), GREEN);
    gui.println(format!("device is {}, destination ip is {}", args.device, args.dstip), WHITE);
    if args.hybrid_mode {
        gui.println(format!("Hash of encryption key : {}", hash_of(layer.layers.encryption_key())), YELLOW);
        gui.println(format!("Hash of your public key: {}", hash_of(
            rsatools::key_as_der(read_file(&args.pubkey_file).unwrap()))), YELLOW);
    }
    gui.println(format!("You can now start writing ...\n"), WHITE);

    gui.input_loop(layer.layers, args.dstip);
}
