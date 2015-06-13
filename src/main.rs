mod logo;
mod humaninterface;
mod humaninterface_std;
mod humaninterface_ncurses;
mod callbacks;

extern crate getopts;
extern crate term;
extern crate icmpmessaging;
extern crate time;

use std::{env, thread};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use getopts::Options;
use term::color;

use icmpmessaging::{Message, IncomingMessage, Errors, Layers};
use humaninterface::{Input, Output};
use callbacks::Callbacks;

#[cfg(not(feature="usencurses"))]
use humaninterface_std::{StdIn, StdOut};
#[cfg(not(feature="usencurses"))]
type HiIn = StdIn;
#[cfg(not(feature="usencurses"))]
type HiOut = StdOut;

#[cfg(feature="usencurses")]
use humaninterface_ncurses::{NcursesIn, NcursesOut};
#[cfg(feature="usencurses")]
type HiIn = NcursesIn;
#[cfg(feature="usencurses")]
type HiOut = NcursesOut;

fn recv_loop(o: Arc<Mutex<HiOut>>, rx: Receiver<IncomingMessage>) {

    thread::spawn(move || { 
        loop { match rx.recv() {
            Ok(msg) => {
                let mut out = o.lock().unwrap();
                match msg {
                    IncomingMessage::New(msg) =>    { out.new_msg(msg); }
                    IncomingMessage::Ack(id)  =>    { out.ack_msg(id); }
                    IncomingMessage::Error(_, s) => { out.err_msg(s); }
                }
            }
            Err(e) => { 
                o.lock().unwrap()
                    .println(format!("recv_loop: failed to receive message. {:?}", e), color::RED); 
            }
        }
    }});
}

fn input_loop(o: Arc<Mutex<HiOut>>, i: HiIn, l: Layers, dstip: String) {

    // read from human interface until user enters control-d and send the
    // message via the network layer
    loop { match i.read_line() {
            Some(s) => {
                let txt = s.trim_right().to_string();
                if txt.len() > 0 {
        		    let msg = Message::new(dstip.clone(), txt.into_bytes());
                    let mut out = o.lock().unwrap();
                    let fm = time::strftime("%R", &time::now()).unwrap();
                    out.println(format!("{} [you] says: {}", fm, s), color::WHITE);
    	        	match l.send(msg) {
    			        Ok(_) => {
                            out.println(format!("transmitting..."), color::BLUE);
    	        		}
    			        Err(e) => { match e {
            				Errors::MessageTooBig => { out.println(format!("main: message too big"), color::RED); }
    	        			Errors::SendFailed => { out.println(format!("main: sending failed"), color::RED); }
                            Errors::EncryptionError => {out.println(format!("main: encryption faild"), color::RED); }
    			        }}
            		}
                }
            }
            _ => { break; }
    }}
    o.lock().unwrap().close();
}


fn main() {
    // parse command line arguments
	let r = parse_arguments();
    let args = if r.is_some() { r.unwrap() } else { return };

    let ret = 
        if args.pub_priv_mode {
            // use asymmetric encryption
            Layers::asymmetric(&args.pubkey_file, &args.privkey_file, &args.device)  // network layer
        } else {
            // use symmetric encryption
            Layers::symmetric(&args.secret_key, &args.device)  // network layer
        };

    if ret.is_err() {
        println!("Initialization failed.");
        return;
    }

    let layer = ret.unwrap();

    let o = Arc::new(Mutex::new(HiOut::new()));    // human interface for output
    let i = HiIn::new();                           // human interface for input

    // this is the loop which handles messages received via rx
    recv_loop(o.clone(), layer.rx);

    {
        let mut out = o.lock().unwrap();
        out.println(logo::get_logo(), color::GREEN);
    	out.println(format!("device is {}, destination ip is {}", args.device, args.dstip), color::WHITE);
	    out.println(format!("You can now start writing ...\n"), color::WHITE);
    }

    input_loop(o.clone(), i, layer.layers, args.dstip);
}

struct Arguments {
    pub device: String,
    pub dstip: String,
    pub pub_priv_mode: bool,
    pub secret_key: String,
    pub pubkey_file: String,
    pub privkey_file: String,
}

fn parse_arguments() -> Option<Arguments> {

    static DEFAULT_SECRET_KEY: &'static str = "11111111111111111111111111111111";

	// parse comand line options
	let args : Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("i", "dev", "set the device where to listen for messages", "device");
	opts.optopt("d", "dst", "set the IP where messages are sent to", "IP");
	opts.optopt("e", "enc", "set the encryption key", "key");
	opts.optopt("r", "recipient", "public key in PEM format used for encryption", "filename");
	opts.optopt("p", "priv", "private key in PEM format used for decryption", "filename");
	opts.optflag("h", "help", "print this message");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!(f.to_string()) }
	};

    let pub_priv_mode = matches.opt_present("r") || matches.opt_present("p");

	if matches.opt_present("h") ||
            (pub_priv_mode && !(matches.opt_present("r") && matches.opt_present("p"))) {
            
		let brief = format!("Usage: {} [options]", args[0]);
		println!("{}", opts.usage(&brief));
		None
	} else {		
        Some(Arguments {
            device: matches.opt_str("i").unwrap_or("lo".to_string()),
            dstip: matches.opt_str("d").unwrap_or("127.0.0.1".to_string()),
            secret_key: matches.opt_str("e").unwrap_or(DEFAULT_SECRET_KEY.to_string()),
            pub_priv_mode: pub_priv_mode,
            pubkey_file: matches.opt_str("r").unwrap_or("".to_string()),
            privkey_file: matches.opt_str("p").unwrap_or("".to_string()),
        })
	}
}



