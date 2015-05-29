mod logo;
mod crypto;

extern crate term;
extern crate getopts;

extern crate icmpmessaging;

use std::env;
use std::io;
use getopts::Options;

use icmpmessaging::network::Message;
use icmpmessaging::network::Network;
use icmpmessaging::network::Errors;
use icmpmessaging::crypto::Encryption;

fn parse_arguments() -> Option<(String, String)> {

	// parse comand line options
	let args : Vec<String> = env::args().collect();

	let mut opts = Options::new();
	opts.optopt("i", "dev", "set the device where to listen for messages", "device");
	opts.optopt("d", "dst", "set the IP where messages are sent to", "IP");
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
		Some((device, dstip))
	}
}

fn println_colored(msg: String, color: term::color::Color) {

    let mut t = term::stdout().unwrap();
    t.fg(color).unwrap();
    (write!(t, "{}", msg)).unwrap();
    t.reset().unwrap();
    (write!(t, "\n")).unwrap();
}

/// This callback function is called when a new message arrives.
fn new_message(msg: Message) {

	let ip = msg.ip;
    let s  = String::from_utf8(msg.buf);
    match s {
        Ok(s)  => { println_colored(format!("{} says: {}", ip, s), term::color::YELLOW); }
        Err(_) => { println!("{} error: could not decode message", ip); }
    }
}

/// This callback function is called when the receiver has received the
/// message with the given id.
///
/// Important notes: Acknowledges are not protected on this layer. An
/// attacker could drop acknowledges or could fake acknowledges. Therefore,
/// it is important that acknowledges are handled on a higher layer where
/// they can be protected via cryptographic mechanisms.
fn ack_message(_id: u64) {

    println_colored("ack".to_string(), term::color::BRIGHT_GREEN);
}

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

fn main() {
    logo::print_logo();

	let r = parse_arguments();
	if r.is_none() {
		return;
	}
	let (device, dstip) = r.unwrap();

	let mut n = Network::new(device.clone(), new_message, ack_message);
    let mut e = init_encryption();

	println!("Device is        : {}", device);
	println!("Destination IP is: {}", dstip);
	println!("\nYou can now start writing ...");

	let mut s = String::new();
	while io::stdin().read_line(& mut s).unwrap() != 0 {
		let msg = Message::new(dstip.clone(), s.trim().to_string().into_bytes());
        if s.trim().len() > 0 {
    		match n.send_msg(msg) {
    			Ok(_) => {
                    println_colored("transmitting...".to_string(), term::color::BLUE);
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
