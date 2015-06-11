mod binding;
mod blowfish;
mod crypto;
mod delivery;
mod packet;
mod rsa;

use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};

use crypto::{Encryption, SymmetricEncryption, AsymmetricEncryption};  // Implemenation for encryption layer
use delivery::Delivery;
use binding::Network;

pub enum IncomingMessage {
    New(Message),
    Ack(u64)
}

unsafe impl Sync for IncomingMessage { } // TODO XXX is it thread safe?
// http://doc.rust-lang.org/std/marker/trait.Sync.html

pub enum MessageType {
    NewMessage,
    AckMessage
}


impl Clone for MessageType {
    fn clone(&self) -> MessageType { 
        match *self {
            MessageType::NewMessage => MessageType::NewMessage,
            MessageType::AckMessage => MessageType::AckMessage
        }
    }
}


pub enum Errors {
	MessageTooBig,
	SendFailed
}


pub struct Message {
    /// Contains the destination ip for outgoing messages, source ip from incoming messages.
	ip : String,
    typ: MessageType,
	buf: Vec<u8>,
}


impl Message {
	pub fn new(ip: String, buf: Vec<u8>) -> Message { Message::create(ip, buf, MessageType::NewMessage) }

	pub fn ack(ip: String) -> Message { Message::create(ip, vec![], MessageType::AckMessage) }

    pub fn set_payload(&self, buf: Vec<u8>) -> Message { 
        Message::create(self.get_ip(), buf, self.get_type())
    }

    pub fn get_payload(&self) -> Vec<u8> { self.buf.clone() }

    /// Returns the destination ip for outgoing messages or the source ip from incoming messages.
    pub fn get_ip(&self) -> String { self.ip.clone() }

    pub fn get_type(&self) -> MessageType { self.typ.clone() }

    fn create(ip: String, buf: Vec<u8>, typ: MessageType) -> Message {
		Message {
			ip: ip,
			buf: buf,
            typ: typ,
		}
	}
}

type EncryptionType = Encryption;

pub struct Layers {
    encryption_layer: Arc<Box<EncryptionType>>,
    delivery_layer  : Delivery
}


impl Layers {
    pub fn symmetric(key: &String, device: &String) -> Option<(Receiver<IncomingMessage>, Layers)> {

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        // network  tx1 --- incoming message ---> rx1 delivery
        // delivery tx2 --- incoming message ---> rx2 layers

        Some(Layers::new(
            Box::new(SymmetricEncryption::new(key)),
            Delivery::new(Network::new(device, tx1), tx2, rx1),
            rx2
        ))
    }

    pub fn asymmetric(pubkey_file: &String, privkey_file: &String, device: &String) -> Option<(Receiver<IncomingMessage>, Layers)> {

        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        // network  tx1 --- incoming message ---> rx1 delivery
        // delivery tx2 --- incoming message ---> rx2 layers

        match AsymmetricEncryption::new(&pubkey_file, &privkey_file) {
            Some(e) =>
                Some(Layers::new(
                    Box::new(e),
                    Delivery::new(Network::new(device, tx1), tx2, rx1),
                    rx2
                )),
            _ => None
        }
    }

    pub fn send(&self, msg: Message) -> Result<u64, Errors> {

        let m = msg.set_payload(self.encryption_layer.encrypt(&msg.buf));
        self.delivery_layer.send_msg(m)
    }

    fn new(e: Box<EncryptionType>, d: Delivery, rx_network: Receiver<IncomingMessage>) -> (Receiver<IncomingMessage>, Layers) {

        // tx is used to send received messages to the application via rx
        let (tx, rx) = channel::<IncomingMessage>();

        let l = Layers {
                    encryption_layer: Arc::new(e),
                    delivery_layer: d,
                };

        l.spawn_receiver(tx.clone(), rx_network);
        (rx, l)
    }

    fn spawn_receiver(&self, tx: Sender<IncomingMessage>, rx: Receiver<IncomingMessage>) {

        let enc = self.encryption_layer.clone();

        thread::spawn(move || { loop { match rx.recv() {
            Ok(msg) => {
                match Layers::handle_message(msg, enc.clone()) {
                    Some(msg) => { 
                        match tx.send(msg) {
                            Err(_) => { println!("error: could not deliver received message to application"); }
                            _ => { }
                        }
                    }
                    _ => { println!("error: could not handle received message") }
                }
            }
            _ => { println!("error: failed to receive message"); }
        }}});
    }

    fn handle_message(m: IncomingMessage, enc: Arc<Box<EncryptionType>>) -> Option<IncomingMessage> {

        match m {
            IncomingMessage::New(msg) => {
                let buf = enc.decrypt(msg.buf.clone());
                match buf {
                    Some(buf) => { Some(IncomingMessage::New(msg.set_payload(buf))) }
                    _ => { None }
                }
            }

            IncomingMessage::Ack(_) => { Some(m) }
        }
    }
}



