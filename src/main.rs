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

pub fn send_msg(layer: Layers, rx: Receiver<String>) {
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(s)  => XXX
                Err(e) => { panic!("failed to receive message: {:?}", e); }
            }
        }
    });
}

fn main() {
	let r = parse_arguments();
    let args = if r.is_some() { r.unwrap() } else { return };

    let layer = match match args.hybrid_mode {
        true  => Layers::asymmetric(&args.rcpt_pubkey_file, &args.privkey_file, &args.device),
        false => Layers::symmetric(&args.secret_key, &args.device)
    } {
        Ok(ly) => ly,
        _ => { panic!("Initialization failed."); }
    };

    let gui = Gui::new();

    gui.println(logo::get_logo(), GREEN);
    gui.println(format!("device is {}, destination ip is {}", args.device, args.dstip), WHITE);
    if args.hybrid_mode {
        gui.println(format!("Hash of encryption key : {}", hash_of(layer.layers.encryption_key())), YELLOW);
        gui.println(format!("Hash of your public key: {}",
            hash_of(rsatools::key_as_der(read_file(&args.pubkey_file).unwrap()))), YELLOW);
    }
    gui.println(format!("You can now start writing ...\n"), WHITE);

    recv_loop(&gui, layer.rx);
    gui.input_loop(layer.layers, args.dstip);
}
