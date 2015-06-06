mod logo;
mod tools;
mod humaninterface;
mod humaninterface_std;
mod humaninterface_ncurses;
mod callbacks;

extern crate getopts;
extern crate term;
extern crate icmpmessaging;

use std::env;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use getopts::Options;
use term::color;

use icmpmessaging::{Message, IncomingMessage, Errors, Layers};
use humaninterface::InputOutput;
use callbacks::Callbacks;
use humaninterface_std::Std;
use humaninterface_ncurses::Ncurses;

type HumanInterface = Std;
//type HumanInterface = Ncurses;


fn recv_loop(rx: Receiver<IncomingMessage>, o: Arc<HumanInterface>) {

    thread::spawn(move || { 
        loop { match rx.recv() {
            Ok(msg) => {
                match msg {
                    IncomingMessage::New(msg) => { o.new_msg(msg); }
                    IncomingMessage::Ack(id)  => { o.ack_msg(id); }
                }
            }
            Err(e)  => { o.println(format!("recv_loop: failed to receive message. {:?}", e), color::RED); }
        }
    }});
}


fn main() {
    logo::print_logo();

    // parse command line arguments
	let r = parse_arguments();
    let (device, dstip, key) = if r.is_some() { r.unwrap() } else { return };

    // initialize human interface for input and output
    let o = Arc::new(HumanInterface::new());

    // initialize the network layer
    let (rx, l) = Layers::default(&key, &device);

    // loop for received messages
    recv_loop(rx, o.clone());

	o.println(format!("device is {}, destination ip is {}", device, dstip), color::WHITE);
	o.println(format!("You can now start writing ...\n"), color::WHITE);

    // read from human interface until user enters control-d and send the
    // message via the network layer
    loop {
        match (*o).read_line() {
            Some(s) => {
                let txt = s.trim_right().to_string();
                if txt.len() > 0 {
        		    let msg = Message::new(dstip.clone(), txt.into_bytes());
    	        	match l.send(msg) {
    			        Ok(_) => {
                            o.println(format!("transmitting..."), color::BLUE);
    	        		}
    			        Err(e) => { match e {
            				Errors::MessageTooBig => { o.println(format!("main: message too big"), color::RED); }
    	        			Errors::SendFailed => { o.println(format!("main: sending failed"), color::RED); }
    			        }}
            		}
                }
            }
            _ => { break; }
        }
    }

    o.quit();
}


fn parse_arguments() -> Option<(String, String, String)> {

    static DEFAULT_ENCRYPTION_KEY: &'static str = "11111111111111111111111111111111";

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



