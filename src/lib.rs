mod binding;
mod delivery;
mod packet;
mod rsa;
mod rsatools;
mod blowfish;
mod cryp;

use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::cryp::{Encryption, SymmetricEncryption, AsymmetricEncryption};  // Implemenation for encryption layer
use crate::delivery::Delivery;
use crate::binding::Network;
use crypto::sha2::Sha256;
use crypto::digest::Digest;

pub enum ErrorType {
    DecryptionError,
    ReceiveError,
}

pub enum IncomingMessage {
    New(Message),
    Ack(u64),
    Error(ErrorType, String),
    FileUpload(Message)
}

unsafe impl Sync for IncomingMessage { } // TODO XXX is it thread safe?
// http://doc.rust-lang.org/std/marker/trait.Sync.html

pub enum MessageType {
    NewMessage,
    AckMessage,
    FileUpload
}


impl Clone for MessageType {
    fn clone(&self) -> MessageType {
        match *self {
            MessageType::NewMessage => MessageType::NewMessage,
            MessageType::AckMessage => MessageType::AckMessage,
            MessageType::FileUpload => MessageType::FileUpload
        }
    }
}


pub enum Errors {
	MessageTooBig,
	SendFailed,
    EncryptionError
}


pub struct Message {
    /// Contains the destination ip for outgoing messages, source ip from incoming messages.
	ip : String,
    typ: MessageType,
	buf: Vec<u8>,
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

impl Message {
    pub fn file_upload(ip: String, fname: String, data: Vec<u8>) -> Message {
        let mut buffer = Vec::from(fname.as_bytes());
        buffer.push(0);
        buffer.extend(data.iter());
        Message::create(ip, buffer, MessageType::FileUpload)
    }

	pub fn new(ip: String, buf: Vec<u8>) -> Message {
        Message::create(ip, buf, MessageType::NewMessage)
    }

	pub fn ack(ip: String) -> Message {
        Message::create(ip, vec![], MessageType::AckMessage)
    }

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

    pub fn sha2(&self) -> String {
        let mut sha2 = Sha256::new();
        sha2.input(&self.buf);
        sha2.result_str()
    }

    pub fn some_bytes(&self) -> Vec<u8> {
        self.buf.iter().take(20).cloned().collect()
    }

    fn create(ip: String, buf: Vec<u8>, typ: MessageType) -> Message {
		Message {
			ip: ip,
			buf: buf,
            typ: typ,
		}
	}
}

pub struct Layer {
    pub rx    : Receiver<IncomingMessage>,
    pub layers: Layers,
}


pub struct Layers {
    encryption_layer: Arc<Box<Encryption>>,
    delivery_layer  : Delivery,
    status_tx       : Sender<String>,
}


impl Layers {

    pub fn symmetric(hexkey: &String, device: &String, status_tx: Sender<String>) -> Result<Layer, &'static str> {

        Layers::init(Box::new(SymmetricEncryption::new(hexkey)?), device, status_tx)
    }

    pub fn asymmetric(pubkey_file: &String, privkey_file: &String, device: &String, status_tx: Sender<String>) -> Result<Layer, &'static str> {

        Layers::init(Box::new(
                AsymmetricEncryption::new(&pubkey_file, &privkey_file)?
            ), device, status_tx
        )
    }

    pub fn send(&self, msg: Message) -> Result<u64, Errors> {

        match self.encryption_layer.encrypt(&msg.buf) {
            Ok(buf) => self.delivery_layer.send_msg(msg.set_payload(buf)),
            _       => Err(Errors::EncryptionError)
        }
    }

    pub fn encryption_key(&self) -> Vec<u8> {
        self.encryption_layer.encryption_key()
    }

    // ------ private functions

    fn init(e: Box<Encryption>, device: &String, status_tx: Sender<String>) -> Result<Layer, &'static str> {

        // network  tx1 --- incoming message ---> rx1 delivery
        // delivery tx2 --- incoming message ---> rx2 layers
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        Ok(Layers::new(e,
            Delivery::new(
                Network::new(device, tx1, status_tx.clone()),
                tx2,
                rx1,
                status_tx.clone(),
            ),
            rx2,
            status_tx
        ))
    }

    fn new(e: Box<Encryption>, d: Delivery, rx_network: Receiver<IncomingMessage>, status_tx: Sender<String>) -> Layer {

        // tx is used to send received messages to the application via rx
        let (tx, rx) = channel::<IncomingMessage>();

        let l = Layers {
            encryption_layer: Arc::new(e),
            delivery_layer: d,
            status_tx: status_tx.clone()
        };

        l.recv_loop(tx, rx_network);
        Layer {
            rx: rx,
            layers: l,
        }
    }

    /// Listens for incoming messages and processes them.
    fn recv_loop(&self, tx: Sender<IncomingMessage>, rx: Receiver<IncomingMessage>) {

        let enc = self.encryption_layer.clone();
        let status_tx = self.status_tx.clone();

        thread::spawn(move || { loop { match rx.recv() {
            Ok(msg) => match Layers::handle_message(msg, enc.clone(), status_tx.clone()) {
                Some(m) => match tx.send(m) {
                    Err(_) => panic!("Channel closed."),
                    _ => { }
                },
                _ => Layers::err(ErrorType::DecryptionError, "Could not decrypt received message.", &tx)
            },
            _ => Layers::err(ErrorType::ReceiveError, "Could not receive message.", &tx)
        }}});
    }

    /// Notifies the application about an error.
    fn err(e: ErrorType, msg: &str, tx: &Sender<IncomingMessage>) {

        match tx.send(IncomingMessage::Error(e, msg.to_string())) {
            Ok(_) => { }
            // If the receiver has hung up quit the application.
            _ => panic!("Channel closed.")
        }
    }

    /// Decrypts incoming messages of type "new" or returns the message without
    /// modification if it is not of type "new".
    fn handle_message(m: IncomingMessage, enc: Arc<Box<Encryption>>, _status_tx: Sender<String>) -> Option<IncomingMessage> {

        // TODO error handling
        #[cfg(feature="debugout")]
        _status_tx.send(String::from("[Layers::handle_message()] decrypting message")).unwrap();

        match m {
            IncomingMessage::New(msg) => {
                #[cfg(feature="debugout")]
                _status_tx.send(format!("[Layers::handle_message()] new message {}", msg.buf.len())).unwrap();

                match enc.decrypt(&msg.buf) {
                    Ok(buf) => Some(IncomingMessage::New(msg.set_payload(buf))),
                    Err(_m) => {
                        #[cfg(feature="debugout")]
                        _status_tx.send(format!("[Layers::handle_message()] decrypt returned with error. {}", _m)).unwrap();
                        None
                    }
                }
            },
            IncomingMessage::FileUpload(msg) => {
                match enc.decrypt(&msg.buf) {
                    Ok(buf) => Some(IncomingMessage::FileUpload(msg.set_payload(buf))),
                    _ => {
                        println!("decryption failed");
                        None
                    }
                }
            },
            IncomingMessage::Ack(_) => Some(m),
            IncomingMessage::Error(_, _) => Some(m)
        }
    }
}


// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    #[test]
    fn test_handle_message() {
    }
}
