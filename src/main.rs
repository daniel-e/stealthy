mod logo;
mod tools;
mod rsatools;
mod options;

extern crate getopts;
extern crate term;
extern crate stealthy;

extern crate crypto as cr;

use cr::sha1::Sha1;
use cr::digest::Digest;

use stealthy::{Message, IncomingMessage, Errors, Layers};
use tools::{read_file, insert_delimiter};
use options::parse_arguments;
use term::color;

mod frontend;
mod globalstate;
use globalstate::GlobalState;
use frontend::humaninterface::Output;

//use rsatools::key_as_der;



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
    let o = gui.o.clone();
    frontend::recv_loop(o.clone(), layer.rx);

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

    let i = gui.i;
    frontend::input_loop(o.clone(), i, layer.layers, args.dstip, &state);
}
