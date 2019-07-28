//use crypto::sha2::Sha256;
//use crypto::digest::Digest;

use crate::error::ErrorType;

unsafe impl Sync for IncomingMessage { } // TODO XXX is it thread safe?
// http://doc.rust-lang.org/std/marker/trait.Sync.html

pub enum IncomingMessage {
    New(Message),
    Ack(u64),
    AckProgress(u64, usize, usize),
    Error(ErrorType, String),
    FileUpload(Message),
}

impl Clone for MessageType {
    fn clone(&self) -> MessageType {
        match *self {
            MessageType::NewMessage => MessageType::NewMessage,
            //MessageType::AckMessage => MessageType::AckMessage,
            MessageType::FileUpload => MessageType::FileUpload
        }
    }
}

pub struct Message {
    /// Contains the destination ip for outgoing messages, source ip from incoming messages.
    pub ip : String,
    pub typ: MessageType,
    pub buf: Vec<u8>,
}

pub enum MessageType {
    NewMessage,
    //AckMessage,
    FileUpload
}

impl Message {
    pub fn file_upload(ip: String, fname: String, data: &Vec<u8>) -> Message {
        let mut buffer = Vec::from(fname.as_bytes());
        buffer.push(0);
        buffer.extend(data.iter());
        Message::create(ip, buffer, MessageType::FileUpload)
    }

    pub fn new(ip: String, buf: Vec<u8>) -> Message {
        Message::create(ip, buf, MessageType::NewMessage)
    }

    /*
    pub fn ack(ip: String) -> Message {
        Message::create(ip, vec![], MessageType::AckMessage)
    }*/

    pub fn set_payload(&self, buf: Vec<u8>) -> Message {
        Message::create(self.get_ip(), buf, self.get_type())
    }

    pub fn get_payload(&self) -> Vec<u8> { self.buf.clone() }

    /// Returns the destination ip for outgoing messages or the source ip from incoming messages.
    pub fn get_ip(&self) -> String { self.ip.clone() }

    pub fn get_type(&self) -> MessageType { self.typ.clone() }

    pub fn get_filename(&self) -> Option<String> {
        let pos = self.get_payload().iter().position(|x| *x == 0 as u8);
        if pos.is_none() {
            // invalid format; TODO error
            return None;
        }
        let payload = self.get_payload();
        let (fname, _) = payload.split_at(pos.unwrap());
        let filename = String::from_utf8(fname.to_vec()).expect("XXXXXXXX"); // TODO error
        Some(sanitize_filename(filename))
    }

    pub fn get_filedata(&self) -> Option<Vec<u8>> {
        let pos = self.get_payload().iter().position(|x| *x == 0 as u8);
        if pos.is_none() {
            // invalid format; TODO error
            return None;
        }
        let payload = self.get_payload();
        let (_, data) = payload.split_at(pos.unwrap() + 1);
        Some(data.to_vec())
    }

    /*
    pub fn sha2(&self) -> String {
        let mut sha2 = Sha256::new();
        sha2.input(&self.buf);
        sha2.result_str()
    }*/

    fn create(ip: String, buf: Vec<u8>, typ: MessageType) -> Message {
        Message {
            ip: ip,
            buf: buf,
            typ: typ,
        }
    }
}

fn replace_char(c: char) -> char {
    match c {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '.' => c,
        _ => '_'
    }
}

fn sanitize_filename(s: String) -> String {
    s.chars().map(|c| replace_char(c)).collect()
}
