mod logo;
mod crypto;
mod tools;

extern crate getopts;
extern crate term;
extern crate icmpmessaging;
extern crate time;

use std::{env, io};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use getopts::Options;
use time::PreciseTime;

use icmpmessaging::network::{Message, Network, Errors, MessageType};
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



struct Layers {
    encryption_layer: Arc<Encryption>,
    network_layer   : Box<Network>,
}

impl Layers {
    fn default(key: &String, device: &String, tx: Sender<Message>) -> Layers {

        let e = Encryption::new(&key);
        let (ltx, lrx) = channel(); // connection between network and layers
        let n = Network::new(device.clone(), ltx);
        Layers::new(e, n, lrx, tx)
    }

    fn new(
        e              : Encryption, 
        n              : Box<Network>,
        rx_from_network: Receiver<Message>,
        tx_application : Sender<Message>) -> Layers {

        let enc = Arc::new(e);
        let enc_thread = enc.clone();

        thread::spawn(move || {
            loop { match rx_from_network.recv() {
                Ok(msg) => {
                    match msg.typ {
                        MessageType::NewMessage => { 
                            match enc_thread.decrypt(msg.buf) {
                                Some(buf) => {
                                    tx_application.send(Message::new(&msg.ip, buf));
                                }

                                None => { println!("{} error: could not decode message", msg.ip) }  // TODO error handling
                            }
                        }
                        MessageType::AckMessage => { tx_application.send(msg); }
                    }
                }
                Err(_) => { println!("Failed to receive message."); }
            }};
        });

        Layers {
            encryption_layer: enc,
            network_layer   : n,
        }
    }

    pub fn send(&mut self, msg: Message) -> Result<u64, Errors> {

        let e = self.encryption_layer.encrypt(msg.buf);
        let m = Message::new(&msg.ip, e); // TODO is this always a new message?

        self.network_layer.send_msg(m)
    }
}


trait Callbacks {
    fn new_msg(&self, msg: Message);
    fn ack_msg(&self, msg: Message);
}


struct MessageHandler;

impl Callbacks for MessageHandler {

    /// This function is called when a new message arrives.
    fn new_msg(&self, msg: Message) {

	    let ip = msg.ip;
        let s  = String::from_utf8(msg.buf);
        let fm = time::strftime("%R", &time::now()).unwrap();

        match s {
            Ok(s)  => { tools::println_colored(format!("[{}] {} says: {}", ip, fm, s), term::color::YELLOW); }
            Err(_) => { 
                tools::println_colored(format!("[{}] {} error: could not decode message", ip, fm), term::color::BRIGHT_RED); 
            }
        }
    }

    /// This callback function is called when the receiver has received the
    /// message with the given id.
    ///
    /// Important notes: Acknowledges are not protected on this layer. An
    /// attacker could drop acknowledges or could fake acknowledges. Therefore,
    /// it is important that acknowledges are handled on a higher layer where
    /// they can be protected via cryptographic mechanisms.
    fn ack_msg(&self, _msg: Message) {

        tools::println_colored("ack".to_string(), term::color::BRIGHT_GREEN);
    }
}

fn init_backend(key: &String, device: &String, handler: MessageHandler) -> (Arc<MessageHandler>, Layers) {

    let (tx, rx) = channel::<Message>();

    let a = Arc::new(handler);
    let h = a.clone();
    let l = Layers::default(&key, &device, tx);

    thread::spawn(move || { 
        loop { match rx.recv() {
            Ok(msg) => {
                match msg.typ {
                    MessageType::NewMessage => { h.new_msg(msg); }
                    MessageType::AckMessage => { h.ack_msg(msg); }
                }
            }
            Err(_)  => { println!("Failed to receive message."); }
        }
    }});

    (a, l)
}


fn main() {
    logo::print_logo();

    // parse command line arguments
	let r = parse_arguments();
    let (device, dstip, key) = if r.is_some() { r.unwrap() } else { return };

    let (mh, mut l) = init_backend(&key, &device, MessageHandler);

	println!("device is {}, destination ip is {}", device, dstip);
	println!("You can now start writing ...\n");

    let mut s = String::new();
    while io::stdin().read_line(&mut s).unwrap() != 0 {
        let txt = s.trim_right().to_string();
        if txt.len() > 0 {
		    let msg = Message::new(&dstip, txt.into_bytes());
    		match l.send(msg) {
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
