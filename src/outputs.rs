use std::iter::repeat;

use crate::{ItemType, Ips};
use crate::Source;
use crate::Arguments;
use crate::Console;

pub fn write_lines(o: Console, lines: &[&str], typ: ItemType, from: Source) {
    for v in lines {
        o.raw(String::from(*v), typ.clone(), from.clone())
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

pub fn help_message(o: Console) {

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

pub struct WelcomeData {
    pub hybrid_mode: bool,
    pub hashed_hybrid_encryption_key: String,
    pub hashed_hybrid_public_key: String,
}

pub fn welcome(args: &Arguments, o: Console, data: WelcomeData, dstips: Ips) {
    for l in get_logo() {
        o.raw(l, ItemType::Introduction, Source::System);
    }

    let ips = dstips.lock().unwrap().as_strings().join(", ");

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

    if data.hybrid_mode {
        o.raw(format!("Hash of encryption key : {}", data.hashed_hybrid_encryption_key), ItemType::Introduction, Source::System);
        o.raw(format!("Hash of your public key: {}", data.hashed_hybrid_public_key), ItemType::Introduction, Source::System);
    }
    o.raw(format!(" "), ItemType::Introduction, Source::System);
    o.raw(format!("Happy chatting..."), ItemType::Introduction, Source::System);
    o.raw(format!(" "), ItemType::Introduction, Source::System);
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
