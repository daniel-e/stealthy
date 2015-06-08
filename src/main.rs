mod logo;
mod tools;
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
                    IncomingMessage::New(msg) => { out.new_msg(msg); }
                    IncomingMessage::Ack(id)  => { out.ack_msg(id); }
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
    			        }}
            		}
                }
            }
            _ => { break; }
    }}
    o.lock().unwrap().close();
}

fn main() {
    logo::print_logo();

    // parse command line arguments
	let r = parse_arguments();
    let (device, dstip, key) = if r.is_some() { r.unwrap() } else { return };

    let o = Arc::new(Mutex::new(HiOut::new()));    // human interface for output
    let i = HiIn::new();                           // human interface for input
    let (rx, l) = Layers::default(&key, &device);  // network layer

    // this is the loop which handles messages received via rx
    recv_loop(o.clone(), rx);

    {
        let mut out = o.lock().unwrap();
    	out.println(format!("device is {}, destination ip is {}", device, dstip), color::WHITE);
	    out.println(format!("You can now start writing ...\n"), color::WHITE);
    }

    input_loop(o.clone(), i, l, dstip);
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



