use crypto::sha1::Sha1;
use crypto::digest::Digest;

use std::iter::repeat;

use crate::ConsoleSender;
use crate::ItemType;
use crate::Source;
use crate::console;
use crate::Layer;
use crate::Arguments;
use crate::IpAddresses;
use crate::rsatools;
use crate::tools::insert_delimiter;
use crate::tools::read_file;

pub fn write_lines(o: ConsoleSender, lines: &[&str], typ: ItemType, from: Source) {
    for v in lines {
        console::raw(o.clone(), String::from(*v), typ.clone(), from.clone())
    }
}

pub fn get_logo() -> Vec<String> {
    // http://patorjk.com/software/taag/#p=display&f=ANSI%20Shadow&t=stealthy
    let mut s = Vec::new();
    s.push(String::from("███████╗████████╗███████╗ █████╗ ██╗  ████████╗██╗  ██╗██╗   ██╗"));
    s.push(String::from("██╔════╝╚══██╔══╝██╔════╝██╔══██╗██║  ╚══██╔══╝██║  ██║╚██╗ ██╔╝"));
    s.push(String::from("███████╗   ██║   █████╗  ███████║██║     ██║   ███████║ ╚████╔╝"));
    s.push(String::from("╚════██║   ██║   ██╔══╝  ██╔══██║██║     ██║   ██╔══██║  ╚██╔╝"));
    s.push(String::from("███████║   ██║   ███████╗██║  ██║███████╗██║   ██║  ██║   ██║"));
    s.push(String::from("╚══════╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚══════╝╚═╝   ╚═╝  ╚═╝   ╚═╝ v0.0.3"));
    s
}

pub fn help_message(o: ConsoleSender) {

    write_lines(o, &vec![
        "Commands always start with a slash:",
        " ",
        "/help                 - this help message",
        "/uptime, /up          - uptime",
        "/cat <filename>       - send content of an UTF-8 encoded text file",
        "/upload <filename>    - send binary file",
        "/set scramble <value> - set timeout in seconds when to scramble content (default: 20)",
        " ",
        "Keys:",
        " ",
        "arrow up     - scroll to older messages",
        "arrow dow    - scroll to latest messages",
        "page up      - scroll one page up",
        "page down    - scroll one page down",
        "end          - scroll to last message in buffer",
        "ctrl+r       - switch to plain messages and back to normal view",
        "ctrl+s       - toggle scrambling",
        "esc | ctrl+d - quit",
        " "
    ], ItemType::Info, Source::System);
}

pub fn welcome(args: &Arguments, o: ConsoleSender, layer: &Layer, dstips: &IpAddresses) {
    for l in get_logo() {
        console::raw(o.clone(), l, ItemType::Introduction, Source::System);
    }

    let ips = dstips.as_strings().join(", ");

    let (values, n) = normalize(&[&args.device, &ips, &ips], ' ');

    let v = vec![
        format!("The most secure ICMP messenger."),
        format!(" "),
        format!("┌─────────────────────┬─{}┐", chars(n, '─')),
        format!("│ Listening on device │ {}│", values[0]),
        format!("│ Talking to IPs      │ {}│", values[1]),
        format!("│ Accepting IPs       │ {}│", values[2]),
        format!("└─────────────────────┴─{}┘", chars(n, '─')),
        format!(" "),
        format!("Type /help to get a list of available commands."),
        format!("Check https://github.com/daniel-e/stealthy for more documentation."),
        format!("Esc or Ctrl+D to quit.")
    ];

    write_lines(
        o.clone(),
        v.iter().map(|x| x.as_str()).collect::<Vec<_>>().as_slice(),
        ItemType::Introduction,
        Source::System
    );

    if args.hybrid_mode {
        let mut h = Sha1::new();

        h.input(&layer.layers.encryption_key());
        let s = insert_delimiter(&h.result_str());
        console::raw(o.clone(), format!("Hash of encryption key : {}", s), ItemType::Introduction, Source::System);

        h.reset();
        h.input(&rsatools::key_as_der(&read_file(&args.pubkey_file).unwrap()));
        let q = insert_delimiter(&h.result_str());
        console::raw(o.clone(), format!("Hash of your public key: {}", q), ItemType::Introduction, Source::System);
    }
    console::raw(o.clone(), format!(" "), ItemType::Introduction, Source::System);
    console::raw(o.clone(), format!("Happy chatting..."), ItemType::Introduction, Source::System);
    console::raw(o.clone(), format!(" "), ItemType::Introduction, Source::System);
}

fn chars(n: usize, c: char) -> String {
    repeat(c).take(n).collect()
}

fn normalize(v: &[&String], c: char) -> (Vec<String>, usize) {
    let maxlen = v.iter().map(|x| x.len()).max().unwrap();
    let r = v.iter()
        .map(|&s| s.clone() + &chars(maxlen - s.len() + 1, c))
        .collect::<Vec<String>>();
    let x = r.iter().map(|s| s.len()).max().unwrap();
    (r, x)
}


// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::get_logo;

    // Just a test for test coverage.
    #[test]
    fn test_get_logo() {
        assert!(get_logo().len() > 5);
    }
}
