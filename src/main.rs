mod logo;
mod tools;
mod options;
mod frontend;
mod globalstate;
mod crypto;
mod blowfish;
mod rsa;
mod rsatools;
mod delivery;
mod binding;
mod packet;

extern crate getopts;
extern crate term;
extern crate stealthy;

extern crate crypto as cr;

use stealthy::{Message, IncomingMessage, Errors, Layers};
use tools::read_file;
use options::parse_arguments;

use globalstate::GlobalState;
use frontend::{Gui, WHITE, GREEN, YELLOW};
use crypto::hash_of;

//use rsatools::key_as_der;

fn main() {
    let state = GlobalState::new();

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
    frontend::recv_loop(gui.o.clone(), layer.rx);

    gui.println(logo::get_logo(), GREEN);
    gui.println(format!("device is {}, destination ip is {}", args.device, args.dstip), WHITE);
    if args.hybrid_mode {
        gui.println(format!("Hash of encryption key : {}", hash_of(layer.layers.encryption_key())), YELLOW);
        gui.println(format!("Hash of your public key: {}", hash_of(
            rsatools::key_as_der(read_file(&args.pubkey_file).unwrap()))), YELLOW);
    }
    gui.println(format!("You can now start writing ...\n"), WHITE);

    gui.input_loop(layer.layers, args.dstip, &state);
}
