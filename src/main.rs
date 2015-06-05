mod logo;
mod tools;
//mod outputcurses;

extern crate getopts;
extern crate term;
extern crate icmpmessaging;
extern crate time;

use std::{env, io};
use std::ops::Drop;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use getopts::Options;
use term::color;

use icmpmessaging::{Message, IncomingMessage, Errors, Layers};


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


trait OutputDevice {
    fn println(&self, s: String, color: color::Color);
}

trait InputDevice {
    fn read_line(&self) -> Option<String>;
}

trait Callbacks {
    fn new_msg(&self, msg: Message);
    fn ack_msg(&self, id: u64);
}


struct Std;

impl OutputDevice for Std {
    fn println(&self, s: String, color: color::Color) {
        tools::println_colored(s, color);
    }
}

impl InputDevice for Std {
    fn read_line(&self) -> Option<String> {
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(n) => {
                if n != 0 { Some(s) } else { None }
            }
            _ => None
        }
    }
}

impl Callbacks for Std {
    /// This function is called when a new message has been received.
    fn new_msg(&self, msg: Message) {

        let ip = msg.get_ip();
        let s  = String::from_utf8(msg.get_payload());
        let fm = time::strftime("%R", &time::now()).unwrap();

        match s {
            Ok(s)  => { self.println(format!("[{}] {} says: {}", ip, fm, s), color::YELLOW); }
            Err(_) => { 
                self.println(format!("[{}] {} error: could not decode message", ip, fm), color::BRIGHT_RED); 
            }
        }
    }

    /// This callback function is called when the receiver has received the
    /// message with the given id.
    ///
    /// Important note: The acknowledge that is received here is the ack on the
    /// network layer which is not protected. An
    /// attacker could drop acknowledges or could fake acknowledges. Therefore,
    /// it is important that acknowledges are handled on a higher layer where
    /// they can be protected via cryptographic mechanisms.
    fn ack_msg(&self, _id: u64) {

        self.println("ack".to_string(), color::BRIGHT_GREEN);
    }
}

fn recv_loop(rx: Receiver<IncomingMessage>, o: Arc<Std>) {

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

    let o = Arc::new(Std);
    let (rx, l) = Layers::default(&key, &device);
    recv_loop(rx, o.clone());

	o.println(format!("device is {}, destination ip is {}", device, dstip), color::WHITE);
	o.println(format!("You can now start writing ...\n"), color::WHITE);

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


}
