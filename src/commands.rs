use crate::ConsoleSender;
use crate::ConsoleMessage;
use crate::Item;
use crate::Layers;
use crate::IpAddresses;
use crate::console;
use crate::ItemType;
use crate::Message;
use crate::Source;
use crate::uptime;
use crate::send_message;
use crate::outputs::help_message;

use crate::tools::{read_file, read_bin_file, decode_uptime, without_dirs};

fn parse_command_set(txt: String, o: ConsoleSender) -> bool {
    let txt_parts = txt.split(' ').collect::<Vec<_>>();
    if !(txt_parts.len() != 3 && txt_parts[1] != "scramble") {
        let n = txt_parts[2].parse::<u32>();
        if n.is_ok() {
            let val = n.unwrap();
            o.send(ConsoleMessage::SetScrambleTimeout(val)).unwrap();
            o.send(ConsoleMessage::TextMessage(Item::new_system(&format!("Value set to {} seconds.", val)))).unwrap();
            return true;
        }
    }
    false
}

pub fn parse_command(txt: String, o: ConsoleSender, l: &Layers, dstips: &IpAddresses) {
    // TODO: find more elegant solution for this
    if txt.starts_with("/cat ") {
        // TODO split_at works on bytes not characters
        let (_, b) = txt.as_str().split_at(5);
        match read_file(b) {
            Ok(data) => {
                console::msg(o.clone(), String::from("Transmitting data ..."), ItemType::Info, Source::System);
                let s = data.as_str();
                for line in s.split("\n") {
                    send_message(line.to_string().trim_end().to_string(), o.clone(), l, dstips);
                }
            },
            _ => {
                console::msg(o.clone(), String::from("Could not read file."), ItemType::Error, Source::System);
            }
        }
        return;
    }

    if txt.starts_with("/set ") {
        if !parse_command_set(txt, o.clone()) {
            o.send(ConsoleMessage::TextMessage(Item::new_system("Command not understood."))).unwrap();
        }
        return;
    }

    if txt.starts_with("/upload ") {
        let (_, b) = txt.as_str().split_at(8);
        match read_bin_file(b) {
            Ok(data) => {
                send_file(data, b.to_string(), o, l, dstips);
            },
            Err(s) => {
                console::msg(o, String::from(s), ItemType::Error, Source::System);
            }
        }
        return;
    }

    match txt.as_str() {
        "/help" => {
            help_message(o.clone());
        },
        "/uptime" | "/up" => {
            console::msg(o, format!("up {}", decode_uptime(uptime())), ItemType::Info, Source::System);
        },
        _ => {
            console::msg(o, String::from("Unknown command. Type /help to see a list of commands."), ItemType::Info, Source::System);
        }
    };
}

fn create_upload_data(dstip: String, fname: &String, data: &Vec<u8>) -> (Message, u64) {
    (
        Message::file_upload(dstip, without_dirs(fname), data),
        rand::random::<u64>()
    )
}

/// Sends a file in background.
///
/// # Arguments
///
/// * `data` - Content of the file (binary data).
/// * `fname` - Name of the file.
/// * `o` - Sender object to which messages are sent to.
fn send_file(data: Vec<u8>, fname: String, console: ConsoleSender, l: &Layers, dstips: &IpAddresses) {

    let n = data.len();

    // This is sent to the console to show the user information about the file upload.
    let mut item = Item::new(
        format!("sending file '{}' with {} bytes...", fname, n),
        ItemType::UploadMessage,
        Source::You
    ).add_size(n);

    // Create a tuple (Message, u64) for each destination IP. For each IP a unique ID is created.
    let v = dstips.as_strings()
        .iter()
        .map(|dstip| create_upload_data(dstip.clone(), &fname, &data))
        .collect::<Vec<_>>();

    // Add the file upload id to the item which is shown to the user. This ID allows us to
    // update the status of this item, e.g. once the file upload is finished.
    for (_, id) in &v {
        item = item.add_id(*id);
    }

    // Show the message.
    console::msg_item(console.clone(),item);

    // Now, start the file transfer in the background for each given IP.
    for (msg, id) in v {
        l.send(msg, id, true);
    }
}
