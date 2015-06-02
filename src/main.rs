mod logo;
mod crypto;
mod tools;

extern crate getopts;
extern crate term;
extern crate icmpmessaging;

use std::env;
use std::io;
use getopts::Options;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;

use icmpmessaging::network::Message;
use icmpmessaging::network::Network;
use icmpmessaging::network::Errors;
use icmpmessaging::network::MessageType;
use icmpmessaging::crypto::Encryption;

static DEFAULT_ENCRYPTION_KEY: &'static str = "11111111111111111111111111111111";

fn parse_arguments() -> Option<(String, String, String)> {

	// parse comand line options
	let args : Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("i", "dev", "set the device where to listen for messages", "device");
	opts.optopt("d", "dst", "set the IP where messages are sent to", "IP");
	opts.optopt("e", "enc", "set the encryption key", "key");
	opts.optflag("h", "help", "print this message");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!(f.to_string()) }
	};

	if matches.opt_present("h") {
		let brief = format!("Usage: {} [options]", args[0]);
		println!("{}", opts.usage(&brief));
		None
	} else {		
		let device = matches.opt_str("i").unwrap_or("lo".to_string());
		let dstip = matches.opt_str("d").unwrap_or("127.0.0.1".to_string());
        let key = matches.opt_str("e").unwrap_or(DEFAULT_ENCRYPTION_KEY.to_string());
		Some((device, dstip, key))
	}
}

/*
fn init_encryption() -> Option<Encryption> {

    // TODO hard coded
    let pubkey_file = "/home/dz/Dropbox/github/icmpmessaging-rs/testdata/rsa_pub.pem";
    let privkey_file = "/home/dz/Dropbox/github/icmpmessaging-rs/testdata/rsa_priv.pem";

    let pubkey = crypto::tools::read_file(pubkey_file);
    let privkey = crypto::tools::read_file(privkey_file);

    match pubkey.is_some() && privkey.is_some() {
        false => {
            println!("Could not read all required keys.");
            None
        }
        true  => { Some(Encryption::new(pubkey.unwrap(), privkey.unwrap())) }
    }
}
*/

struct MessageHandle {
    e: Arc<Encryption>
}

impl MessageHandle {

    pub fn new(e: Arc<Encryption>) -> MessageHandle {
        MessageHandle {
            e: e
        }
    }

    /// This function is called when a new message arrives.
    fn new_msg(&self, msg: Message) {

	    let ip = msg.ip;

        match (*self.e).decrypt(msg.buf) {
            Some(buf) => {
                let s  = String::from_utf8(buf);
                match s {
                    Ok(s)  => { tools::println_colored(format!("{} says: {}", ip, s), term::color::YELLOW); }
                    Err(_) => { println!("{} error: could not decode message", ip); }
                }
            }

            None => { println!("{} error: could not decode message", ip) }
        }
    }

    /// This callback function is called when the receiver has received the
    /// message with the given id.
    ///
    /// Important notes: Acknowledges are not protected on this layer. An
    /// attacker could drop acknowledges or could fake acknowledges. Therefore,
    /// it is important that acknowledges are handled on a higher layer where
    /// they can be protected via cryptographic mechanisms.
    fn ack_msg(&self, msg: Message) {

        tools::println_colored("ack".to_string(), term::color::BRIGHT_GREEN);
    }
}

fn recv_loop(rx: Receiver<Message>, mh: Arc<Mutex<MessageHandle>>) {

    thread::spawn(move || { 
        let message_handling = mh.clone();
        loop { match rx.recv() {
            Ok(msg) => {
                let x = message_handling.lock().unwrap();
                match msg.typ {
                    MessageType::NewMessage => { x.new_msg(msg); }
                    MessageType::AckMessage => { x.ack_msg(msg); }
                }
            }
            Err(_)  => { println!("Failed to receive message."); }
        }
    }});
}

fn main() {
    logo::print_logo();

    // parse command line arguments
	let r = parse_arguments();
	if r.is_none() {
		return;
	}
	let (device, dstip, key) = r.unwrap();

    let (tx, rx) = channel();
    let e        = Arc::new(Encryption::new(&key));
    let mh       = Arc::new(Mutex::new(MessageHandle::new(e.clone())));

    recv_loop(rx, mh.clone());

	let mut n = Network::new(device.clone(), tx);

	println!("Device is        : {}", device);
	println!("Destination IP is: {}", dstip);
	println!("\nYou can now start writing ...");

    let mut s = String::new();
    while io::stdin().read_line(&mut s).unwrap() != 0 {
        let txt = s.trim().to_string();
		let msg = Message::new(dstip.clone(), (*e).encrypt(txt.into_bytes()), MessageType::NewMessage);
        if s.trim().len() > 0 {
    		match n.send_msg(msg) {
    			Ok(_) => {
                    tools::println_colored("transmitting...".to_string(), term::color::BLUE);
    			}
    			Err(e) => { match e {
    				Errors::MessageTooBig => { println!("main: message too big"); }
    				Errors::SendFailed => { println!("main: sending failed"); }
    			}}
    		}
        }
		s.clear();
	}
}
