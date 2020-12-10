use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;
use getopts::Options;

pub struct Arguments {
    pub device: String,
    pub dstip: String,
    pub hybrid_mode: bool,
    pub secret_key: String,
    pub rcpt_pubkey_file: String,
    pub privkey_file: String,
    pub pubkey_file: String,
    pub ranges: Vec<String>,
}

fn get_key_from_home() -> Option<String> {
    match dirs::home_dir() {
        Some(mut path) => {
            path.push(".stealthy/key");
            match File::open(path) {
                Ok(f) => {
                    let mut reader = BufReader::new(f);
                    let mut key = String::new();
                    match reader.read_line(&mut key) {
                        Ok(_) => Some(key.trim().to_string()),
                        _ => None
                    }
                }
                _ => None
            }
        },
        None => None
    }
}

pub fn parse_arguments() -> Option<Arguments> {

    static DEFAULT_SECRET_KEY: &'static str = "11111111111111111111111111111111";

    // parse comand line options
    let args : Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("i", "dev", "set the device where to listen for messages", "device");
    opts.optopt("d", "dst", "set the IP where messages are sent to", "IP");
    opts.optopt("e", "enc", "set the encryption key", "key");
    opts.optopt("r", "recipient", "recipient's public key in PEM format used for encryption", "filename");
    opts.optopt("p", "priv", "your private key in PEM format used for decryption", "filename");
    opts.optopt("q", "pub", "your public key in PEM format", "filename");
    opts.optmulti("b", "probe", "in case destination is not available probe the given range", "IP range");
    opts.optflag("h", "help", "print this message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let hybrid_mode = matches.opt_present("r") || matches.opt_present("p");

    if matches.opt_present("h") ||
        (hybrid_mode && !(matches.opt_present("r") && matches.opt_present("p") && matches.opt_present("q"))) {

        let brief = format!("Usage: {} [options]", args[0]);
        println!("{}", opts.usage(&brief));
        return None;
    }

    // 1) If option -e is given use this key.
    // 2) If key exists in home directory use this key.
    // 3) Use default key.
    let key = matches.opt_str("e")
        .unwrap_or(get_key_from_home().unwrap_or(DEFAULT_SECRET_KEY.to_string()));

    Some(Arguments {
        device:       matches.opt_str("i").unwrap_or("lo".to_string()),
        dstip:        matches.opt_str("d").unwrap_or("127.0.0.1".to_string()),
        secret_key:   key,
        hybrid_mode:  hybrid_mode,
        rcpt_pubkey_file:  matches.opt_str("r").unwrap_or("".to_string()),
        privkey_file: matches.opt_str("p").unwrap_or("".to_string()),
        pubkey_file:  matches.opt_str("q").unwrap_or("".to_string()),
        ranges: matches.opt_strs("b"),
    })
}
